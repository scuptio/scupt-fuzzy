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
    Initialize(Message<String>),
    MessageReq(Message<String>),
}

impl MsgTrait for FuzzyCommand {

}

impl FuzzyCommand {
    pub fn command_type(&self) -> FuzzyCmdType {
        match self {
            FuzzyCommand::Initialize(_) => { FuzzyCmdType::Initialize }
            FuzzyCommand::MessageReq(_) => { FuzzyCmdType::MessageReq }
        }
    }
}

pub enum FuzzyCmdType {
    Initialize,
    MessageReq,
}
