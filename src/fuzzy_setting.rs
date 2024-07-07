use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};

#[derive(
    Serialize,
    Deserialize,
    Clone)]
pub struct FuzzySetting {
    /// return the possible crash/restart payload text
    pub crash_restart_payload: Vec<(String, String)>,

    pub restart_after_max_ms:u64,

    /// return the node id set
    pub node:  Vec<NID>,

    /// maximum duplicated count

    pub message_max_duplicated:u64,
    /// maximum delayed milliseconds

    pub message_max_delay_ms: u64,

    pub crash_ratio:f64,

    pub network_partition_ratio:f64,

    pub partition_end_after_max_ms:u64,

    pub message_delay_ratio:f64,

    pub message_repeat_ratio:f64,

    pub message_lost_ratio:f64,
}


