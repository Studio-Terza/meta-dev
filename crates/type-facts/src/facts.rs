mod collector;
mod import;
mod members;

use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::ops::Not;

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::{CrateName, ModulePath};

use crate::generic_impl::GenericImpl;
use crate::implementation::Implementation;
use crate::method::Method;
use crate::outside::Outside;
use crate::trait_fact::TraitFact;
use crate::type_fact::TypeFact;
use crate::vantage::Vantage;
use collector::{Collector, ImplBlock};
use import::Import;
pub(crate) use members::Members;

#[derive(Clone, Debug, Default)]
pub struct Facts<'file> {
    types: Vec<TypeFact<'file>>,
    tested_enums: Vec<TypeFact<'file>>,
    imports: Vec<Import<'file>>,
    traits: Vec<TraitFact<'file>>,
    generic_impls: Vec<GenericImpl<'file>>,
    members: BTreeMap<&'file CrateName, BTreeMap<Name, Members<'file>>>,
}

impl<'file> Facts<'file> {
    #[must_use]
    pub fn of_files(files: &'file [SourceFile]) -> Facts<'file> {
        files.iter().filter_map(SourceFile::shipped).fold(Facts::default(), Collector::collected)
    }

    pub(crate) fn add_type(&mut self, found: Option<TypeFact<'file>>) {
        self.types.extend(found);
    }

    pub(crate) fn add_tested(&mut self, found: Vec<TypeFact<'file>>) {
        self.tested_enums.extend(found);
    }

    pub(crate) fn add_imports(&mut self, found: Vec<Import<'file>>) {
        self.imports.extend(found);
    }

    pub(crate) fn add_trait(&mut self, found: Option<TraitFact<'file>>) {
        self.traits.extend(found);
    }

    pub(crate) fn add_generic(&mut self, found: Option<GenericImpl<'file>>) {
        self.generic_impls.extend(found);
    }

    pub(crate) fn add_block(&mut self, found: Option<ImplBlock<'file>>) {
        self.members = found.into_iter().fold(mem::take(&mut self.members), |members, block| block.added_to(members));
    }

    #[must_use]
    pub fn types(&self) -> &[TypeFact<'file>] {
        &self.types
    }

    #[must_use]
    pub fn declared_once<'found>(&self, found: &'found TypeFact<'file>) -> Option<&'found TypeFact<'file>> {
        (self.types.iter().filter_map(|other| other.named_like(found)).count() == 1).then_some(found)
    }

    #[must_use]
    pub fn enums_named(&self, vantage: &Vantage<'_>, name: &Name) -> Vec<&TypeFact<'file>> {
        let shipped = Self::enums_of(&self.types, vantage, name);
        let tested = Self::enums_of(&self.tested_enums, vantage, name);
        let reachable: Vec<&TypeFact<'file>> =
            vantage.gating().pick(shipped.clone(), shipped.iter().chain(&tested).copied().collect());
        let here =
            reachable.iter().copied().filter(|found| AsRef::<ModulePath>::as_ref(*found) == vantage.module()).collect();
        let brought = reachable.iter().copied().filter(|found| self.brought(vantage, found).is_some()).collect();
        let own = vantage.gating().pick(shipped, tested);
        [here, brought, own].into_iter().find(|found| found.is_empty().not()).unwrap_or_default()
    }

    fn enums_of<'found>(
        found: &'found [TypeFact<'file>],
        vantage: &Vantage<'_>,
        name: &Name,
    ) -> Vec<&'found TypeFact<'file>> {
        found
            .iter()
            .filter(|each| AsRef::<CrateName>::as_ref(*each) == vantage.krate())
            .filter(|each| AsRef::<Name>::as_ref(*each) == name)
            .filter_map(TypeFact::as_enum)
            .collect()
    }

    fn brought(&self, vantage: &Vantage<'_>, found: &TypeFact<'file>) -> Option<&Import<'file>> {
        self.imports
            .iter()
            .filter_map(|import| import.made_in(vantage.krate(), vantage.module()))
            .find_map(|import| import.bringing(found))
    }

    #[must_use]
    pub fn traits(&self) -> &[TraitFact<'file>] {
        &self.traits
    }

    #[must_use]
    pub fn generic_impls(&self) -> &[GenericImpl<'file>] {
        &self.generic_impls
    }

    #[must_use]
    pub fn implementers(&self, implemented: &Name) -> Vec<&Name> {
        self.members
            .values()
            .flat_map(BTreeMap::iter)
            .filter(|(_, members)| {
                members.implementations().iter().any(|each| AsRef::<Name>::as_ref(each) == implemented)
            })
            .map(|(name, _)| name)
            .collect()
    }

    pub fn outside_implementations(&self) -> impl Iterator<Item = Outside<'_, 'file>> + '_ {
        let declared: BTreeSet<(&CrateName, &Name)> =
            self.types.iter().map(|found| (AsRef::as_ref(found), AsRef::as_ref(found))).collect();
        let named = self.members.iter().flat_map(|(krate, named)| named.iter().map(move |each| (*krate, each)));
        named
            .filter(move |(krate, (name, _))| declared.contains(&(*krate, *name)).not())
            .flat_map(|(_, (name, members))| members.implementations().iter().map(move |each| Outside::new(name, each)))
    }

    #[must_use]
    pub fn methods(&self, found: &TypeFact<'file>) -> &[Method] {
        self.members_of(found).map_or(&[], Members::methods)
    }

    #[must_use]
    pub fn implementations(&self, found: &TypeFact<'file>) -> &[Implementation<'file>] {
        self.members_of(found).map_or(&[], Members::implementations)
    }

    fn members_of(&self, found: &TypeFact<'file>) -> Option<&Members<'file>> {
        let krate: &CrateName = found.as_ref();
        let name: &Name = found.as_ref();
        self.members.get(krate)?.get(name)
    }
}
