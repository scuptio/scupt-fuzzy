use scupt_util::message::Message;
use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};



#[derive(
Clone,
Debug,
Serialize,
Deserialize
)]
pub enum FuzzyEvent {
    /// delay some milliseconds
    Delay(u64, Message<String>),

    /// message was lost
    Lost,

    /// duplicate message
    Duplicate(Vec<u64>, Message<String>),

    Crash(Message<String>),
    
    Restart(u64, Message<String>),

    PartitionStart(Vec<NID>, Vec<NID>),

    PartitionRecovery(u64, Vec<NID>, Vec<NID>)
}