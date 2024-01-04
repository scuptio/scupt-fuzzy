use scupt_util::message::Message;
use scupt_util::serde_json_string::SerdeJsonString;


pub trait  Initializer : Clone {
    fn initialize_message(&self) -> Vec<Message<SerdeJsonString>>;
}

#[derive(Clone)]
pub struct InitializerPhantom {

}

impl Initializer for InitializerPhantom {
    fn initialize_message(&self) -> Vec<Message<SerdeJsonString>> {
        vec![]
    }
}