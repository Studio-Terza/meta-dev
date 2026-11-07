use crate::Deadline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Job {
    deadline: Deadline,
}

impl Job {
    #[must_use]
    pub fn due_by(deadline: Deadline) -> Job {
        Job { deadline }
    }

    #[must_use]
    pub fn deadline(&self) -> Deadline {
        self.deadline
    }
}
