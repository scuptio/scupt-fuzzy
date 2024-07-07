use bincode::{Decode, Encode};
use scupt_util::message::{Message, MsgTrait};
use serde::{Deserialize, Serialize};

#[derive(
Clone,
Hash,
PartialEq,
Eq,
Debug,
Serialize,
Deserialize,
Decode,
Encode,
)]
pub enum FuzzyCommand {

    MessageReq(Message<String>),
}

impl MsgTrait for FuzzyCommand {}

impl FuzzyCommand {
    pub fn command_type(&self) -> FuzzyCmdType {
        match self {
            FuzzyCommand::MessageReq(_) => { FuzzyCmdType::MessageReq }
        }
    }
}

pub enum FuzzyCmdType {
    MessageReq,
}
