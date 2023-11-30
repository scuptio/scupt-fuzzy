

use std::sync::Arc;
use scupt_net::client::{Client, OptClientConnect};
use scupt_net::notifier::Notifier;
use scupt_util::error_type::ET;
use scupt_util::message::{Message, MsgTrait};
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use scupt_util::serde_json_string::SerdeJsonString;

pub struct  FuzzyClient {
    client: Arc<Client<SerdeJsonString>>,
}


impl FuzzyClient {
    pub fn new(node_id:NID, name:String, addr:String, notifier:Notifier) -> Res<Self> {
        let client =  Client::<SerdeJsonString>::new(node_id, name, addr, notifier)?;
        Ok(Self {
            client: Arc::new(client)
        })
    }

    async fn connect(&self) -> Res<()> {
        let mut opt = OptClientConnect::new();
        opt.retry_max = u64::MAX;
        opt.retry_wait_ms = 1000;
        self.client.connect(opt).await?;
        Ok(())
    }

    pub async fn send<M:MsgTrait + 'static>(&self, message: Message<M>) -> Res<()> {
        let source = message.source();
        let dest = message.dest();
        let s = SerdeJsonString::new(
            serde_json::to_string_pretty(&message).unwrap());
        let r = self.client.send(Message::new(s.clone(), source, dest)).await;
        match r {
            Ok(()) => { }
            Err(e) => {
                if ET::NetNotConnected == e {
                    self.connect().await?;
                    self.client.send(Message::new(s.clone(), source, dest)).await?;
                }
            }
        }
        Ok(())
    }
}

