use crate::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    uuid: Uuid,
}

impl Skipped {
    #[must_use]
    pub fn new(uuid: Uuid) -> Skipped {
        Skipped { uuid }
    }

    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}
