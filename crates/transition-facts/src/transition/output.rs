use crate::endpoint::Endpoint;

#[derive(Clone, Copy, Debug)]
pub struct Output<'found>(&'found Endpoint);

impl<'found> Output<'found> {
    pub(crate) const fn new(endpoint: &'found Endpoint) -> Self {
        Output(endpoint)
    }

    #[must_use]
    pub const fn endpoint(self) -> &'found Endpoint {
        self.0
    }
}
