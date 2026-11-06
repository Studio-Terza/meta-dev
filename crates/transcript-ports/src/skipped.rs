use crate::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    uuid: Uuid,
    parent: Option<Uuid>,
}

impl Skipped {
    pub(crate) fn new(uuid: Uuid, parent: Option<Uuid>) -> Skipped {
        Skipped { uuid, parent }
    }

    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    #[must_use]
    pub fn parent(&self) -> Option<&Uuid> {
        self.parent.as_ref()
    }
}
