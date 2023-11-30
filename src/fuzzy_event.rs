use serde::{Deserialize, Serialize};



#[derive(
Clone,
Debug,
Serialize,
Deserialize
)]
pub enum FuzzyEvent {
    /// delay some milliseconds
    Delay(u64),

    /// message was lost
    Lost,

    /// duplicate message
    Duplicate(Vec<u64>),

    Crash,
    
    Restart(u64)
}