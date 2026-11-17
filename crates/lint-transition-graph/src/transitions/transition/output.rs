use crate::transitions::endpoint::Endpoint;
use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub struct Output<'found>(&'found Endpoint);

impl<'found> Output<'found> {
    #[must_use]
    pub fn endpoint(self) -> &'found Endpoint {
        self.0
    }
}
