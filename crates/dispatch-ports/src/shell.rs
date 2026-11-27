mod boundary;

use std::collections::BTreeMap;
use std::sync::Mutex;

use invariants::Unpoisoned;
use port_core::{Pass, port};
use serde::Deserialize;

use crate::DispatchFailure;
pub use boundary::{Call, Printed, Spoken};

#[port]
pub trait Shell {
    fn ran(&self, call: &Call) -> Result<Ran, DispatchFailure>;
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Ran {
    Succeeded(Printed),
    Failed(Printed),
    Overdue,
}

impl Ran {
    #[must_use]
    pub fn printed(&self) -> Option<&Printed> {
        match self {
            Ran::Succeeded(printed) | Ran::Failed(printed) => Some(printed),
            Ran::Overdue => None,
        }
    }

    #[must_use]
    pub fn succeeded(&self) -> Option<&Printed> {
        match self {
            Ran::Succeeded(printed) => Some(printed),
            Ran::Failed(_) | Ran::Overdue => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct AnsweringShell {
    answers: BTreeMap<Spoken, Vec<Ran>>,
    asked: Mutex<Vec<Call>>,
}

impl AnsweringShell {
    #[must_use]
    pub fn answering(answers: BTreeMap<Spoken, Vec<Ran>>) -> AnsweringShell {
        AnsweringShell { answers, asked: Mutex::default() }
    }

    #[must_use]
    pub fn asked(&self) -> Vec<Call> {
        self.asked.lock().unpoisoned().clone()
    }
}

impl ShellFiller for &AnsweringShell {
    fn ran(&self, _pass: Pass<'_, shell_ops::Ran>, call: &Call) -> Result<Ran, DispatchFailure> {
        let spoken = Spoken::from(call);
        let mut asked = self.asked.lock().unpoisoned();
        let earlier = asked.iter().filter(|before| Spoken::from(*before) == spoken).count();
        asked.push(call.clone());
        let answers = self.answers.get(&spoken).ok_or(DispatchFailure::Unanswered)?;
        answers.get(earlier).or(answers.last()).cloned().ok_or(DispatchFailure::Unanswered)
    }
}
