use gate_limits::Stage;
use serde::Deserialize;

use super::super::number::Number;

#[derive(Debug, Deserialize)]
pub struct Listed {
    number: Number,
    title: Stage,
}

impl Listed {
    pub fn numbered(&self, stage: &Stage) -> Option<Number> {
        (self.title == *stage).then_some(self.number)
    }
}
