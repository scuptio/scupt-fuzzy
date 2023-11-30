use scupt_util::node_id::NID;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(
Clone,
Serialize,
Debug,
Deserialize,
)]
pub struct ServerConfig {
    pub db_path:String,
    pub node_id:NID,
    pub net_address:String,
    pub peers : HashMap<NID, String>,
}