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
    fn on_handle(&self, message:Message<String>);
}

lazy_static! {
    static ref EVENT_HANDLER : ConcurrentHashMap<String, Arc<dyn  FEventMsgHandler>> = ConcurrentHashMap::new();
    static ref FUZZY : ConcurrentHashMap<String, FuzzyClient> = ConcurrentHashMap::new();
    static ref EVENT:ConcurrentHashMap<String, Arc<Mutex<Vec<SerdeJsonValue>>>> =
        ConcurrentHashMap::new();
    static ref INJECTED_EVENT:ConcurrentHashMap<String, Arc<Mutex<Vec<SerdeJsonValue>>>> =
        ConcurrentHashMap::new();
}



pub fn event_sequence_setup(s:&str, v:Vec<String>) {
    let v = v.iter().map(|_s|{
        SerdeJsonValue::new(serde_json::from_str(_s.as_str()).unwrap())
    }).collect();
    let _ = INJECTED_EVENT.insert(s.to_string(), Arc::new(Mutex::new(v)));
    let _ = EVENT.insert(s.to_string(), Arc::new(Mutex::new(vec![])));
}

pub fn event_sequence_get(s:&str) -> Option<Vec<String>> {
    let e = EVENT.get(&s.to_string())?;
    let g = e.get().lock().unwrap();
    let vec = g.iter().map(|_s|{
        _s.to_serde_json_string().to_string()
    }).collect();
    Some(vec)
}

pub fn event_sequence_unset(s:&str) {
    let _ = EVENT.remove(&s.to_string());
    let _ = INJECTED_EVENT.remove(&s.to_string());
}

pub fn event_sequence_add<M:MsgTrait + 'static>(s:&str, e:Message<M>) {
    let opt1 = EVENT_HANDLER.get(&s.to_string());
    match opt1 {
        Some(v) => {
            let _m = e.clone();
            let h1 = v.get().clone();
            h1.on_handle(_m.map(|m| {
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
    let sequence = match opt {
        Some(e) => {
            let s = e.get().clone();
            let mut seq = s.lock().unwrap();
            seq.push(sjv);
            (*seq).clone()
        }
        None => { return; }
    };

    let opt1 = INJECTED_EVENT.get(&name);
    if let Some(e) = opt1 {
        let g = e.get().lock().unwrap();
        if g.eq(&sequence) {

        }
    }
}

/// Event sequence setup
#[macro_export]
macro_rules! event_seq_setup {
    ($name:expr, $vec:expr) => {
        {
            scupt_fuzzy::fuzzy::event_sequence_setup($name, $vec);
        }
    };
}

#[macro_export]
macro_rules! event_seq_get {
    ($name:expr) => {
        {
            scupt_fuzzy::fuzzy::event_sequence_get($name);
        }
    };
}
/// Event sequence unset
#[macro_export]
macro_rules! event_seq_unset {
    ($name:expr) => {
        {
            scupt_fuzzy::fuzzy::fuzzy_testing_unset($name);
        }
    };
}

/// Event sequence unset
#[macro_export]
macro_rules! event_seq_add {
    ($name:expr, $message:expr) => {
        {
            scupt_fuzzy::fuzzy::event_sequence_add($name, $message);
        }
    };
}

pub fn fuzzy_message_event_handle_setup(name:&str, handle:Arc<dyn FEventMsgHandler>) {
    let _ = EVENT_HANDLER.insert(name.to_string(), handle);
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
}

pub async fn fuzzy_testing_message<M:MsgTrait + 'static>(name:&str, message:Message<M>) {
    let opt = FUZZY.get(&name.to_string());
    match opt {
        Some(v) => {
            let _ = v.get().send(message).await;
        }
        None => {

        }
    }
}

/// Fuzzy testing setup
#[macro_export]
macro_rules! fuzzy_event_handler_setup {
    ($name:expr, $handler:expr) => {
        {
            scupt_fuzzy::fuzzy::fuzzy_message_event_handle_setup($name, $handler);
        }
    };
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