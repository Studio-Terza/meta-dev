use lint_layout::Name;
use lint_placement::CrateName;

use super::super::member::Member;

#[derive(Debug)]
pub(crate) struct Target<'found> {
    krate: &'found CrateName,
    module: Option<Vec<Name>>,
    here: Option<&'found [Name]>,
}

impl<'found> Target<'found> {
    pub(crate) fn new(
        krate: &'found CrateName,
        module: Option<Vec<Name>>,
        here: Option<&'found [Name]>,
    ) -> Target<'found> {
        Target { krate, module, here }
    }

    pub(crate) fn unhinted(self) -> Target<'found> {
        Target { module: None, ..self }
    }

    pub(crate) fn krate(&self) -> &'found CrateName {
        self.krate
    }

    pub(crate) fn declared(&self, declaring: &[Member<'found>]) -> Option<Member<'found>> {
        let candidates: Vec<Member<'found>> =
            declaring.iter().copied().filter(|member| member.cluster().krate() == Some(self.krate)).collect();
        let only = candidates.first().copied().filter(|_| candidates.len() == 1);
        let unique = |found: Vec<Member<'found>>| found.first().copied().filter(|_| found.len() == 1);
        let segments = |member: &Member<'found>| member.cluster().segments();
        let named = self.module.as_ref().and_then(|module| {
            let exact = candidates.iter().copied().filter(|held| segments(held) == module.as_slice()).collect();
            let under = candidates
                .iter()
                .copied()
                .filter(|held| segments(held).get(..module.len()) == Some(module.as_slice()))
                .collect();
            unique(exact).or_else(|| unique(under))
        });
        let local = self
            .here
            .and_then(|here| unique(candidates.iter().copied().filter(|held| segments(held) == here).collect()));
        only.or(named).or(local)
    }
}
