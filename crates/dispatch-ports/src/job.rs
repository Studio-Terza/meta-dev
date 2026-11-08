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
}

impl AsRef<Deadline> for Job {
    fn as_ref(&self) -> &Deadline {
        &self.deadline
    }
}
