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
    Message(Message<String>)
}

impl MsgTrait for FuzzyCommand {

}