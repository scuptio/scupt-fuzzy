use std::collections::HashMap;

use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};

#[derive(
Clone,
Serialize,
Debug,
Deserialize,
)]
pub struct ServerConfig {
    pub db_path: String,
    pub node_id: NID,
    pub net_address: String,
    pub peers: HashMap<NID, String>,
}