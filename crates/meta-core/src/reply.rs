use crate::event::Event;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Accepted {
    pub events: Vec<Event>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(String);

impl Refusal {
    pub fn because(why: impl Into<String>) -> Self {
        Self(why.into())
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "refused: {}", self.0)
    }
}
