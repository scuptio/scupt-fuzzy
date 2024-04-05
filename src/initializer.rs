use scupt_util::message::Message;
use scupt_util::serde_json_string::SerdeJsonString;

pub trait Initializer: Send + Sync {
    fn message(&self) -> Vec<Message<SerdeJsonString>>;
}

#[derive(Clone)]
pub struct InitializerPhantom {}

impl Default for InitializerPhantom {
    fn default() -> Self {
        Self {}
    }
}

impl Initializer for InitializerPhantom {
    fn message(&self) -> Vec<Message<SerdeJsonString>> {
        vec![]
    }
}