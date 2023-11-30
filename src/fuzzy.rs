use lazy_static::lazy_static;
use scc::HashMap as ConcurrentHashMap;
use scupt_net::notifier::Notifier;
use scupt_util::message::{Message, MsgTrait};
use scupt_util::node_id::NID;
use crate::fuzzy_client::FuzzyClient;

lazy_static! {
    static ref FUZZY : ConcurrentHashMap<String, FuzzyClient> = ConcurrentHashMap::new();
}


pub fn fuzzy_testing_setup(name:&str, id:NID, addr:String) {
    let name = name.to_string();
    let client = FuzzyClient::new(id, name.clone(), addr, Notifier::new()).unwrap();
    let _ = FUZZY.insert(name, client);
}

pub fn fuzzy_testing_unset(name:&str) {
    let _ = FUZZY.remove(&name.to_string());
}

pub async fn fuzzy_testing_message<M:MsgTrait + 'static>(name:&str, message:Message<M>) {
    let opt = FUZZY.get(&name.to_string());
    match opt {
        Some(v) => {
            v.get().send(message).await.unwrap();
        }
        None => {

        }
    }
}

/// Fuzzy testing setup
#[macro_export]
macro_rules! fuzzy_setup {
    ($name:expr, $id:expr, $addr:expr) => {
        {
            fuzzy::fuzzy_testing_setup($name, $id, $addr);
        }
    };
}

/// Fuzzy testing unset
#[macro_export]
macro_rules! fuzzy_unset {
    ($name:expr, $id:expr, $addr:expr) => {
        {
            fuzzy::fuzzy_testing_unset($name);
        }
    };
}
/// Is an automation enable
#[macro_export]
macro_rules! fuzzy_message {
    ($name:expr, $message:expr) => {
        {
            fuzzy::fuzzy_testing_message($name, $message)
        }
    };
}