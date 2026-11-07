use crate::endpoint::Endpoint;

#[derive(Clone, Copy, Debug)]
pub struct Implementer<'found>(&'found Endpoint);

impl<'found> Implementer<'found> {
    pub(crate) fn new(endpoint: &'found Endpoint) -> Self {
        Implementer(endpoint)
    }

    #[must_use]
    pub fn endpoint(self) -> &'found Endpoint {
        self.0
    }
}
