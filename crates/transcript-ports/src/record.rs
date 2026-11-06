use crate::{Authorship, Body, Instant, Uuid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub uuid: Uuid,
    pub parent: Option<Uuid>,
    pub session: Uuid,
    pub at: Instant,
    pub author: Authorship,
    pub text: Body,
}
