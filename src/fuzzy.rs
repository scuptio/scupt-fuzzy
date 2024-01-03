use std::any::Any;

use lazy_static::lazy_static;
use scc::HashMap as ConcurrentHashMap;
use scupt_net::notifier::Notifier;
use scupt_util::message::{Message, MsgTrait};
use scupt_util::node_id::NID;
use crate::fuzzy_client::FuzzyClient;
use std::sync::{Arc, Mutex};

use std::vec::Vec;
use scupt_util::serde_json_value::SerdeJsonValue;

pub trait FEventMsgHandler : Send + Sync + Any {
    fn on_handle(&self, name:String, message:Message<String>);
}

lazy_static! {
    static ref EVENT_HANDLER : ConcurrentHashMap<String, Arc<dyn  FEventMsgHandler>> = ConcurrentHashMap::new();
    static ref FUZZY : ConcurrentHashMap<String, FuzzyClient> = ConcurrentHashMap::new();
    static ref EVENT:ConcurrentHashMap<String, Arc<Mutex<Vec<SerdeJsonValue>>>> =
        ConcurrentHashMap::new();
}



pub fn event_sequence_setup(s:&str, handle:Arc<dyn FEventMsgHandler>) {
    let _ = EVENT.insert(s.to_string(), Arc::new(Mutex::new(vec![])));
    let _ = EVENT_HANDLER.insert(s.to_string(), handle);
}

pub fn event_sequence_unset(s:&str) {
    let _ = EVENT.remove(&s.to_string());
}

pub fn event_sequence_add<M:MsgTrait + 'static>(s:&str, e:Message<M>) {
    let opt1 = EVENT_HANDLER.get(&s.to_string());
    match opt1 {
        Some(v) => {
            let _m = e.clone();
            let h1 = v.get().clone();
            h1.on_handle(s.to_string(), _m.map(|m| {
                serde_json::to_string(&m).unwrap()
            }))
        }
        None => {

        }
    }
    let name = s.to_string();
    let opt = EVENT.get(&name);
    let v = serde_json::to_value(&e).unwrap();
    let sjv = SerdeJsonValue::new(v);
    match opt {
        Some(e) => {
            let s = e.get().clone();
            let mut seq = s.lock().unwrap();
            seq.push(sjv);

        }
        None => { return; }
    };
}

/// Event sequence setup
#[macro_export]
macro_rules! event_setup {
    ($name:expr, $handler:expr) => {
        {
            scupt_fuzzy::fuzzy::event_sequence_setup($name, $handler);
        }
    };
}


/// Event sequence unset
#[macro_export]
macro_rules! event_unset {
    ($name:expr) => {
        {
            scupt_fuzzy::fuzzy::fuzzy_testing_unset($name);
        }
    };
}

/// Event sequence unset
#[macro_export]
macro_rules! event_add {
    ($name:expr, $message:expr) => {
        {
            scupt_fuzzy::fuzzy::event_sequence_add($name, $message);
        }
    };
}

pub fn fuzzy_testing_setup(name:&str, id:NID, addr:String) {
    let name = name.to_string();
    let client = FuzzyClient::new(id, name.clone(), addr, Notifier::new()).unwrap();
    let _ = FUZZY.insert(name, client);
}

pub fn fuzzy_testing_enable(name:&str) -> bool {
    FUZZY.contains(&name.to_string())
}

pub fn fuzzy_testing_unset(name:&str) {
    let _ = FUZZY.remove(&name.to_string());
    let _ = EVENT.remove(&name.to_string());
    let _ = EVENT_HANDLER.remove(&name.to_string());
}

pub async fn fuzzy_testing_message<M:MsgTrait + 'static>(name:&str, message:Message<M>) {
    let opt = FUZZY.get(&name.to_string());
    match opt {
        Some(v) => {
            let _ = v.get().fuzzy_rpc(message).await;
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
            scupt_fuzzy::fuzzy::fuzzy_testing_setup($name, $id, $addr);
        }
    };
}

/// Fuzzy testing unset
#[macro_export]
macro_rules! fuzzy_unset {
    ($name:expr) => {
        {
            scupt_fuzzy::fuzzy::fuzzy_testing_unset($name);
        }
    };
}
/// Is an automation enable
#[macro_export]
macro_rules! fuzzy_message {
    ($name:expr, $message:expr) => {
        {
            scupt_fuzzy::fuzzy::fuzzy_testing_message($name, $message).await;
        }
    };
}

#[macro_export]
macro_rules! fuzzy_enable {
    ($name:expr) => {
        {
            scupt_fuzzy::fuzzy::fuzzy_testing_enable($name)
        }
    };
}