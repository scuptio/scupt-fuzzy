use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;


use scupt_net::event_sink::{ESConnectOpt, ESServeOpt, EventSink};

use scupt_net::io_service::{IOService, IOServiceOpt};
use scupt_net::message_receiver::ReceiverRR;



use scupt_net::notifier::Notifier;
use scupt_net::task::spawn_local_task;
use scupt_util::error_type::ET;
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use scupt_util::serde_json_string::SerdeJsonString;
use tokio::runtime::Builder;
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
    service_message_to_nodes:IOService<SerdeJsonString>,
    service_message_incoming:IOService<FuzzyCommand>,
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
        let service_to_nodes = IOService::new(
            nid,
            format!("service_to_node_{}", name.clone()),
            opt1,
            notify.clone())?;
        let server_service_incoming = IOService::new(
            nid,
            format!("service_incoming_{}", name.clone()),
            opt2,
            notify.clone())?;
        let sender_to_node = service_to_nodes.default_message_sender();
        let inner = Self {
            peers,
            server_addr,
            notifier : notify.clone(),
            fuzzy_driver: Arc::new(
                FuzzyDriver::new(
                path,
                notify.clone(),
                sender_to_node,
                event_generator
            )),
            service_message_to_nodes: service_to_nodes,
            service_message_incoming: server_service_incoming,
        };
        inner.fuzzy_driver.create_db()?;
        Ok(inner)
    }

    fn run(&self) {
        let n1 = self.notifier.clone();
        let n2 = self.notifier.clone();
        let n3 = self.notifier.clone();
        let ls1 = LocalSet::new();
        self.run_server(&ls1);
        let ls2 = LocalSet::new();
        self.service_message_incoming.run_local(&ls2);
        let ls3 = LocalSet::new();
        self.service_message_to_nodes.run_local(&ls3);
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let ls = LocalSet::new();
        ls.spawn_local(async move {
            for (name, n, ls) in [
                ("server loop", n1, ls1),
                ("service incoming", n2, ls2),
                ("service to nodes", n3, ls3)] {
                let _ = spawn_local_task(n, name, async move {
                    ls.await;
                });
            }
        });
        runtime.block_on(async move {
            ls.await
        });
    }



    fn run_server(&self, ls:&LocalSet) {
        let server_sink_message_incoming = self.service_message_incoming.default_event_sink();
        let server_address = self.server_addr.clone();


        let client_connect_to_peers = self.peers.clone();
        let server_sink_connect_to_node = self.service_message_to_nodes.default_event_sink();

        let notifier1 = self.notifier.clone();
        let driver = self.fuzzy_driver.clone();
        let receiver = self.service_message_incoming.message_receiver_rr();
        // let receiver = self.service_message_incoming.message_receiver();

        let notifier = self.notifier.clone();
        ls.spawn_local(async move {
            let _ = spawn_local_task(notifier, "server_start", async move {
                Self::server_serve(server_sink_message_incoming, server_address).await?;
                Self::server_connect_to_all_tested_nodes(server_sink_connect_to_node, client_connect_to_peers).await?;
                Self::server_handle_recv_message(notifier1, driver, receiver).await?;
                Ok::<(), ET>(())
            });

        });
    }

    async fn server_serve(
        sink:Arc<dyn EventSink>,
        server_address:SocketAddr,
    ) ->Res<()>{
        sink.serve(server_address, ESServeOpt::new().enable_no_wait(false)).await?;
        Ok(())
    }

    pub async fn server_connect_to_all_tested_nodes(
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
        receiver:Vec<Arc<dyn ReceiverRR<FuzzyCommand>>>
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
