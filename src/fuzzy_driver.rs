
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use rusqlite::Connection;
use scupt_net::message_receiver::Receiver;
use scupt_net::message_sender::Sender;
use scupt_net::notifier::Notifier;
use scupt_net::opt_send::OptSend;
use scupt_net::task::spawn_local_task;
use scupt_util::error_type::ET;
use scupt_util::message::{Message, message_dest_id, message_source_id};

use scupt_util::res::Res;
use scupt_util::res_of::res_sqlite;
use scupt_util::serde_json_string::SerdeJsonString;
use serde_json::to_string_pretty;
use tokio::time::sleep;
use crate::fuzzy_command::FuzzyCommand;
use crate::fuzzy_event::FuzzyEvent;
use crate::fuzzy_generator::FuzzyGenerator;

pub struct FuzzyDriver<F:FuzzyGenerator + 'static>  {
    path_store: String,
    notifier : Notifier,
    event_generator : F,
    inner:Arc<FuzzyInner>,
}

struct FuzzyInner {
    atomic_sequence: AtomicU64,
    sender: Arc<dyn Sender<SerdeJsonString>>,
    path:String,
}

impl <F:FuzzyGenerator + 'static> FuzzyDriver<F> {
    pub fn new(path:String, sender:Arc<dyn Sender<SerdeJsonString>>, event_generator:F) -> Self {
        Self {
            path_store: path.clone(),
            notifier: Default::default(),
            event_generator,

            inner: Arc::new(FuzzyInner { atomic_sequence: AtomicU64::new(0), sender, path }),
        }
    }

    pub fn create_db(&self) -> Res<()> {
        let mut conn = Connection::open(self.path_store.clone()).unwrap();
        let trans = res_sqlite(conn.transaction())?;
        let _r = trans.execute(
            r#"create table action (
                    id interger primary key,
                    message text not null,
                    event text not null
                );"#, ());
        res_sqlite(_r)?;
        let _r = trans.execute (
            r#"create table dilivery (
                    id interger primary key,
                    action_id integer not null
                );"#, ());
        res_sqlite(_r)?;
        Ok(())
    }



    pub async fn message_loop(
        &self,
        receiver:Arc<dyn Receiver<FuzzyCommand>>) -> Res<()> {
        loop {
            let command = receiver.receive().await?;
            self.incoming_command(command.payload()).await?;
        }
    }

    pub async fn incoming_command(&self, command:FuzzyCommand) -> Res<()>{
        let seq =  self.event_generator.gen(command);
        for (event, command) in seq {
            match command {
                FuzzyCommand::Message(message) => {
                    let id = self.inner.gen_id();
                    self.fuzzy_event_for_message(id, event, message).await?;
                }
            }
        }
        Ok(())
    }

    fn store_event_message(&self, id:u64, event:FuzzyEvent, message:SerdeJsonString) {
        let mut conn = Connection::open(self.path_store.clone()).unwrap();
        let transaction = conn.transaction().unwrap();
        let event_s = serde_json::to_string_pretty(&event).unwrap();
        let message_s = to_string_pretty(message.to_serde_json_value().serde_json_value_ref()).unwrap();
        let _ = transaction.execute("\
                            upsert message(id, message, event) \
                            values(?1, ?2, ?3)", (&id, &message_s, &event_s)).unwrap();

        transaction.commit().unwrap();
    }

    async fn fuzzy_event_for_message(&self, id:u64, event:FuzzyEvent, message:SerdeJsonString) -> Res<()> {
        self.store_event_message(id, event.clone(), message.clone());
        self.schedule_fuzzy_event(id, event, message).await?;
        Ok(())
    }

    async fn schedule_fuzzy_event(&self, id:u64, event:FuzzyEvent, message:SerdeJsonString) -> Res<()>{
        let inner = self.inner.clone();
        let _ = spawn_local_task(self.notifier.clone(), "", async move {
            inner.schedule(id, event, message).await?;
            Ok::<(), ET>(())
        })?;
        Ok(())
    }
}

impl FuzzyInner {
    async fn schedule(&self, id:u64, event:FuzzyEvent, message:SerdeJsonString) -> Res<()> {
        match event {
            FuzzyEvent::Delay(ms) => {
                sleep(Duration::from_millis(ms)).await;
                self.send(id, message).await?;
            }
            FuzzyEvent::Duplicate(vec) => {
                for ms in vec {
                    sleep(Duration::from_millis(ms)).await;
                    self.send(id, message.clone()).await?;
                }
            }
            FuzzyEvent::Lost => {}
            FuzzyEvent::Crash => {
                self.send(id, message.clone()).await?;
            }
        }
        Ok(())
    }

    async fn send(&self, id:u64, message:SerdeJsonString) -> Res<()> {
        self.store_message_delivery(id);
        let value = message.to_serde_json_value();
        let source = message_source_id(&value).unwrap();
        let dest = message_dest_id(&value).unwrap();
        let m = Message::new(message, source, dest);
        let _= self.sender.send(m, OptSend::default()).await?;
        Ok(())
    }

    fn gen_id(&self) -> u64 {
        let id = self.atomic_sequence.fetch_add(1, Ordering::SeqCst);
        return id
    }

    fn store_message_delivery(&self, action_id:u64) {
        let mut conn = Connection::open(self.path.clone()).unwrap();
        let id = self.gen_id();
        let transaction = conn.transaction().unwrap();
        let _ = transaction.execute("\
                            upsert delivery(id, action_id) \
                            values(?1, ?2, ?3)", (&id, &action_id)).unwrap();

        transaction.commit().unwrap();
    }

}