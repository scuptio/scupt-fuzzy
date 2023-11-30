use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;


use scupt_net::event_sink::{ESConnectOpt, ESServeOpt, EventSink};

use scupt_net::io_service::{IOService, IOServiceOpt};
use scupt_net::message_receiver::Receiver;



use scupt_net::notifier::Notifier;
use scupt_net::task::spawn_local_task;
use scupt_util::error_type::ET;
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use scupt_util::serde_json_string::SerdeJsonString;
use tokio::task::LocalSet;
use tokio::time::sleep;
use tracing::trace;
use crate::fuzzy_command::FuzzyCommand;
use crate::fuzzy_driver::FuzzyDriver;
use crate::fuzzy_generator::FuzzyGenerator;

pub struct FuzzyServer<F:FuzzyGenerator + 'static> {
    inner:Arc<FuzzyServerInner<F>>,
}

struct FuzzyServerInner<F:FuzzyGenerator + 'static>  {
    peers:HashMap<NID, SocketAddr>,
    server_addr:SocketAddr,
    notifier:Notifier,
    fuzzy_driver:Arc<FuzzyDriver<F>>,
    client_service:IOService<SerdeJsonString>,
    server_service:IOService<FuzzyCommand>,
}


impl <F:FuzzyGenerator + 'static> FuzzyServer<F> {
    pub fn new(
        nid:NID,
        name:String,
        path:String,
        notifier:Notifier,
        server_addr:SocketAddr,
        peers: HashMap<NID, SocketAddr>,
        event_generator:F,
    ) -> Res<Self> {
        let r = Self {
            inner: Arc::new(
                FuzzyServerInner::new(
                    nid,
                    name,
                    path,
                    notifier,
                    server_addr,
                    peers,
                    event_generator)?),
        };
        Ok(r)
    }

    pub fn run(&self)  {
        self.inner.run();
    }
}

impl <F:FuzzyGenerator> FuzzyServerInner<F> {
    fn new(nid:NID,
           name:String,
           path:String,
           notify:Notifier,
           server_addr:SocketAddr,
           peers: HashMap<NID, SocketAddr>,
           event_generator:F,
           ) -> Res<Self> {
        let opt1 = IOServiceOpt {
            num_message_receiver: 1,
        };
        let opt2 = IOServiceOpt {
            num_message_receiver: 1,
        };
        let client_service = IOService::new(nid, name.clone(), opt1, notify.clone())?;
        let server_service = IOService::new(nid, name.clone(), opt2, notify.clone())?;
        let sender = client_service.default_message_sender();
        let inner = Self {
            peers,
            server_addr,
            notifier : notify.clone(),
            fuzzy_driver: Arc::new(FuzzyDriver::new(
                path, sender,
                event_generator
            )),
            client_service,
            server_service,
        };
        inner.fuzzy_driver.create_db()?;
        Ok(inner)
    }

    fn run(&self) {
        let ls = LocalSet::new();
        self.run_server(&ls);
        self.server_service.run_local(&ls);
        self.client_service.run_local(&ls);
    }



    fn run_server(&self, ls:&LocalSet) {
        let server_sink = self.server_service.default_event_sink();
        let server_address = self.server_addr.clone();


        let client_connect_to_peers = self.peers.clone();
        let client_sink = self.client_service.default_event_sink();

        let notifier1 = self.notifier.clone();
        let driver = self.fuzzy_driver.clone();
        let receiver = self.server_service.message_receiver();

        let notifier = self.notifier.clone();
        ls.spawn_local(async move {
            let _ = spawn_local_task(notifier, "server_start", async move {
                Self::server_serve(server_sink, server_address).await?;
                Self::client_connect_all(client_sink, client_connect_to_peers).await?;
                Self::server_handle_recv_message(notifier1, driver, receiver).await?;
                Ok::<(), ET>(())
            });

        });
    }

    async fn server_serve(
        sink:Arc<dyn EventSink>,
        server_address:SocketAddr,
    ) ->Res<()>{
        sink.serve(server_address, ESServeOpt::new()).await?;
        Ok(())
    }

    pub async fn client_connect_all(
        sink:Arc<dyn EventSink>,
        node_address: HashMap<NID, SocketAddr>,
    ) -> Res<()> {
        let mut connected = HashSet::new();
        loop {
            for (id, addr) in node_address.iter() {
                let r = sink.connect(
                    id.clone(), addr.clone(),
                    ESConnectOpt::new()
                        .enable_no_wait(false)
                        .enable_return_endpoint(false),
                ).await;
                match r {
                    Ok(_) => {
                        connected.insert(id.clone());
                    }
                    Err(_e) => {
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            if node_address.len() == connected.len() {
                break;
            }
        }
        trace!("serve player, connect to all");
        Ok(())
    }

    async fn server_handle_recv_message(
        notifier: Notifier,
        fuzzy_driver: Arc<FuzzyDriver<F>>,
        receiver:Vec<Arc<dyn Receiver<FuzzyCommand>>>
    ) -> Res<()> {
        for r in receiver {
            let driver = fuzzy_driver.clone();
            let _ = spawn_local_task(notifier.clone(), "", async move {
                driver.message_loop(r).await?;
                Ok::<(), ET>(())
            })?;
        }
        Ok(())
    }
}
