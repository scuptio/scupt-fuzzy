use async_trait::async_trait;
use crate::fuzzy_command::FuzzyCommand;
use crate::fuzzy_event::FuzzyEvent;

#[async_trait]
pub trait  FuzzyGenerator : Clone + Sync + Send {
    async fn gen(&self, cmd:FuzzyCommand) -> Vec<FuzzyEvent>;
}


