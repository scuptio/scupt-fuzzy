use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::serde_json_string::SerdeJsonString;
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
    Message(SerdeJsonString)
}

impl MsgTrait for FuzzyCommand {

}