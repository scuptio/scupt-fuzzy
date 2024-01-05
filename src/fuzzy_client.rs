use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use scupt_net::client::{Client, OptClientConnect};
use scupt_net::notifier::Notifier;
use scupt_util::message::{Message, MsgTrait};
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use tokio::runtime;
use tokio::task::LocalSet;

use crate::fuzzy_command::{FuzzyCmdType, FuzzyCommand};

pub struct  FuzzyClient {
    inner: Arc<FuzzyClientInner>
}

pub struct  FuzzyClientInner {
    client :Client<FuzzyCommand>,
    _join_handler: JoinHandle<()>,
}

impl FuzzyClient {
    pub fn new(node_id:NID, name:String, addr:String, notifier:Notifier) -> Res<Self> {
        let inner = FuzzyClientInner::new(node_id, name, addr, notifier)?;
        Ok(Self {
            inner: Arc::new(inner)
        })
    }

    pub async fn fuzzy_rpc<M: MsgTrait + 'static>(&self, cmd_type: FuzzyCmdType, message: Message<M>) -> Res<()> {
        self.inner.fuzzy_rpc(cmd_type, message).await?;
        Ok(())
    }
}

impl FuzzyClientInner {
    pub fn new(node_id:NID, name:String, addr:String, notifier:Notifier) -> Res<Self> {
        let client =  Client::new(node_id, name, addr, notifier)?;
        let c = client.clone();
        let join_handler = thread::Builder::new().spawn(move || {
            let ls = LocalSet::new();
            c.run(&ls);
            let j = runtime::Builder::new_current_thread().enable_all().build().unwrap();
            j.block_on(async move {
                ls.await;
            });
        }).unwrap();

        Ok(Self {
            client,
            _join_handler: join_handler,
        })

    }

    async fn connect(&self) -> Res<()> {
        let mut opt = OptClientConnect::new();
        opt.retry_max = 1;
        opt.retry_wait_ms = 1000;
        let r = self.client.connect(opt).await;
        match r {
            Ok(()) => { Ok(()) }
            Err(e) => { Err(e) }
        }
    }


    async fn fuzzy_rpc<M: MsgTrait + 'static>(&self, cmd_type: FuzzyCmdType, message: Message<M>) -> Res<()> {
        let source = message.source();
        let dest = message.dest();
        let json_string = serde_json::to_string_pretty(&message).unwrap();
        let fuzzy_command = match cmd_type {
            FuzzyCmdType::Initialize => {
                FuzzyCommand::Initialize(Message::new(json_string, source, dest))
            }
            FuzzyCmdType::MessageReq => {
                FuzzyCommand::MessageReq(Message::new(json_string, source, dest))
            }
        };
        if !self.client.is_connected().await {
            self.connect().await?;
        }
        self.client.send(Message::new(fuzzy_command, source, dest)).await?;
        Ok(())
    }
}

