use crate::fuzzy_command::FuzzyCommand;
use crate::fuzzy_event::FuzzyEvent;

pub trait  FuzzyGenerator {
    fn gen(&self, cmd:FuzzyCommand) -> Vec<(FuzzyEvent, FuzzyCommand)>;
}


