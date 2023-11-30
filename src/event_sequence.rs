use std::sync::{Arc, Mutex};
use std::vec;
use scc::HashMap as ConcurrentHashMap;
use lazy_static::lazy_static;
use vec::Vec;
use scupt_util::message::{Message, MsgTrait};
use scupt_util::serde_json_value::SerdeJsonValue;

lazy_static! {
    static ref EVENT:ConcurrentHashMap<String, Arc<Mutex<Vec<SerdeJsonValue>>>> =
        ConcurrentHashMap::new();
    static ref INJECTED_EVENT:ConcurrentHashMap<String, Arc<Mutex<Vec<SerdeJsonValue>>>> =
        ConcurrentHashMap::new();
}

pub fn event_sequence_setup(s:&str, v:Vec<SerdeJsonValue>) {
    let _ = INJECTED_EVENT.insert(s.to_string(), Arc::new(Mutex::new(v)));
    let _ = EVENT.insert(s.to_string(), Arc::new(Mutex::new(vec![])));
}

pub fn event_sequence_unset(s:&str) {
    let _ = EVENT.remove(&s.to_string());
    let _ = INJECTED_EVENT.remove(&s.to_string());
}

pub fn event_sequence_add<M:MsgTrait + 'static>(s:&str, e:Message<M>) {
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
            panic!("injected error");
        }
    }
}

/// Event sequence setup
#[macro_export]
macro_rules! event_seq_setup {
    ($name:expr, $vec:expr) => {
        {
            scupt_fuzzy::event_sequence::event_sequence_setup($name, $vec);
        }
    };
}


/// Event sequence unset
#[macro_export]
macro_rules! event_seq_unset {
    ($name:expr) => {
        {
            scupt_fuzzy::event_sequence::fuzzy_testing_unset($name);
        }
    };
}

/// Event sequence unset
#[macro_export]
macro_rules! event_seq_add {
    ($name:expr, $message:expr) => {
        {
            scupt_fuzzy::event_sequence::event_sequence_add($name, $message);
        }
    };
}