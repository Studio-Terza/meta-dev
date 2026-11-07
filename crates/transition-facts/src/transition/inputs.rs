use crate::endpoint::Endpoint;
use crate::resolved::Resolved;

#[derive(Clone, Copy, Debug)]
pub struct Inputs<'found> {
    resolved: &'found [Resolved],
    selfish: &'found Endpoint,
}

impl<'found> Inputs<'found> {
    pub(crate) fn new(resolved: &'found [Resolved], selfish: &'found Endpoint) -> Self {
        Inputs { resolved, selfish }
    }

    pub fn iter(self) -> impl Iterator<Item = &'found Endpoint> {
        self.resolved.iter().map(move |taken| taken.resolved_with(self.selfish))
    }
}
