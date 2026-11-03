use crate::endpoint::Endpoint;

#[derive(Clone, Copy, Debug)]
pub struct Output<'found>(&'found Endpoint);

impl<'found> Output<'found> {
    pub(crate) fn new(endpoint: &'found Endpoint) -> Output<'found> {
        Output(endpoint)
    }

    #[must_use]
    pub fn endpoint(self) -> &'found Endpoint {
        self.0
    }
}
