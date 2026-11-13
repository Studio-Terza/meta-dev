use std::collections::BTreeMap;

use code_model::{ItemId, Model, References};
use lint_core::Site;
use lint_item_graph::Traversal;
use lint_layout::Name;

use super::link::Link;
use super::member::Member;
use super::resolver::StandIns;

#[derive(Debug)]
pub(crate) struct Bodies<'found> {
    model: &'found Model,
    members: BTreeMap<ItemId, Member<'found>>,
    crates: BTreeMap<ItemId, &'found Name>,
}

impl<'found> Bodies<'found> {
    pub(crate) fn placed<'file>(
        model: &'found Model,
        declared: &BTreeMap<Site<'file>, Vec<Member<'found>>>,
    ) -> Bodies<'found> {
        let members = model
            .items()
            .filter_map(|placed| {
                let item = *placed.entity();
                let name: &Name = model.declared(item)?.as_ref();
                let member = declared.get(&placed.site())?.iter().find(|member| member.name() == name)?;
                Some((item, *member))
            })
            .collect();
        let crates = model.items().map(|placed| (*placed.entity(), placed.krate())).collect();
        Bodies { model, members, crates }
    }

    pub(crate) fn links(&self, references: &References<'found>) -> Vec<Link<'found>> {
        let targets = references.targets();
        let standing = |item: ItemId| {
            let kind = self.model.declared(item).map(code_model::Item::kind);
            kind.filter(|found| StandIns::STANDING.contains(found)).map(|_| item)
        };
        let next = |item: ItemId| {
            let krate = self.crates.get(&item);
            let used = standing(item).and_then(|found| targets.get(&found)).into_iter().flatten().copied();
            used.filter(move |target| self.crates.get(target) == krate)
        };
        targets
            .iter()
            .filter_map(|(owner, used)| {
                Some(self.linked(*self.members.get(owner)?, Traversal::trees(used.iter().copied(), &next)))
            })
            .flatten()
            .collect()
    }

    fn linked(&self, holder: Member<'found>, reached: Vec<Vec<ItemId>>) -> Vec<Link<'found>> {
        let ends = reached.into_iter().flatten().filter_map(|end| self.members.get(&end));
        ends.map(|member| Link::new(holder, *member)).collect()
    }
}
