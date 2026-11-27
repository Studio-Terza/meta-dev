mod boundary;

use crate::Deadline;

pub use boundary::Model;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Job {
    deadline: Deadline,
    model: Model,
}

impl Job {
    #[must_use]
    pub fn due_by(deadline: Deadline, model: Model) -> Job {
        Job { deadline, model }
    }
}

impl AsRef<Deadline> for Job {
    fn as_ref(&self) -> &Deadline {
        &self.deadline
    }
}

impl AsRef<Model> for Job {
    fn as_ref(&self) -> &Model {
        &self.model
    }
}
