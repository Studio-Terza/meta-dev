use super::super::member::Member;
use super::super::unit::Unit;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Target<'found> {
    unit: Unit<'found>,
    module: Option<Vec<Name>>,
    here: Option<&'found [Name]>,
}

impl<'found> Target<'found> {
    pub(crate) fn new(unit: Unit<'found>, module: Option<Vec<Name>>, here: Option<&'found [Name]>) -> Target<'found> {
        Target { unit, module, here }
    }

    pub(crate) fn unhinted(self) -> Target<'found> {
        Target { module: None, ..self }
    }

    pub(crate) fn unit(&self) -> Unit<'found> {
        self.unit
    }

    pub(crate) fn declared(&self, declaring: &[Member<'found>]) -> Option<Member<'found>> {
        let candidates: Vec<Member<'found>> =
            declaring.iter().copied().filter(|member| member.cluster().unit() == Some(self.unit)).collect();
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
