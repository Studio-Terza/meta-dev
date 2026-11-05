use std::collections::BTreeMap;
use std::path::Path;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Transitions;

use super::super::cluster::Cluster;
use super::super::member::Member;

#[derive(Debug, Default)]
pub(crate) struct Functions<'found>(BTreeMap<&'found Path, BTreeMap<&'found Name, Member<'found>>>);

impl<'found> Functions<'found> {
    pub(crate) fn declared<'file>(
        transitions: &'found Transitions<'file>,
        homes: &BTreeMap<&Path, Cluster<'found>>,
    ) -> Functions<'found> {
        let free = transitions.all().filter(|transition| transition.context().subject().is_none());
        free.fold(Functions::default(), |functions, transition| {
            let site: &'found Site<'file> = transition.function().as_ref();
            let krate: &'found CrateName = transition.context().as_ref();
            let name: &'found Name = transition.function().as_ref();
            let member = homes.get(site.path()).map(|home| Member::declared(krate, name).within(*home));
            functions.declaring(site.path(), member)
        })
    }

    fn declaring(mut self, file: &'found Path, member: Option<Member<'found>>) -> Functions<'found> {
        self.0.entry(file).or_default().extend(member.map(|member| (member.name(), member)));
        self
    }

    pub(crate) fn in_file(&self, file: &Path) -> BTreeMap<&'found Name, Member<'found>> {
        self.0.get(file).cloned().unwrap_or_default()
    }
}
