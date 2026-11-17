use crate::transitions::endpoint::Endpoint;
use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub struct Implementer<'found>(&'found Endpoint);

impl<'found> Implementer<'found> {
    #[must_use]
    pub fn endpoint(self) -> &'found Endpoint {
        self.0
    }
}
