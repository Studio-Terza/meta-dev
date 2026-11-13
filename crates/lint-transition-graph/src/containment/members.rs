use std::collections::{BTreeMap, BTreeSet};

use code_model::{
    Function, Holding, Implementation, ItemDeclaration, ItemId, ItemKind, Model, Named, Origin, Placed, Reach,
    References, Region, Resolved, Seat, Sited, Traversal, TypeRef, TypeUses,
};
use lint_core::Site;
use lint_layout::Name;

use super::link::Link;
use super::member::Member;
use super::outsider::Outsider;

#[derive(Debug, Default)]
struct Reached(BTreeSet<ItemId>);

impl TypeUses for Reached {
    fn named(&mut self, named: &Named<Sited>) {
        self.0.extend(AsRef::<Resolved>::as_ref(named).crate_item());
        named.each_inner_use(self);
    }
}

impl<'ty> FromIterator<&'ty TypeRef> for Reached {
    fn from_iter<Types: IntoIterator<Item = &'ty TypeRef>>(types: Types) -> Reached {
        types.into_iter().fold(Reached::default(), |mut reached, ty| {
            ty.each_use(&mut reached);
            reached
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct Declared<'file> {
    item: ItemId,
    member: Member<'file>,
    site: Site<'file>,
}

#[derive(Debug)]
pub(crate) struct Members<'file> {
    model: &'file Model,
    references: References<'file>,
    declarations: BTreeMap<ItemId, Member<'file>>,
    sites: BTreeMap<Member<'file>, Site<'file>>,
    crates: BTreeMap<ItemId, &'file Name>,
}

impl<'file> From<&'file Model> for Members<'file> {
    fn from(model: &'file Model) -> Members<'file> {
        let declared: Vec<Placed<'file, &'file ItemDeclaration>> = model.items().collect();
        let found: Vec<Declared<'file>> = declared.iter().filter_map(|placed| Self::member(model, *placed)).collect();
        Members {
            model,
            references: References::from(model),
            declarations: found.iter().map(|each| (each.item, each.member)).collect(),
            sites: found.iter().map(|each| (each.member, each.site)).collect(),
            crates: declared.iter().map(|placed| (placed.entity().declared(), placed.krate())).collect(),
        }
    }
}

impl<'file> Members<'file> {
    pub(crate) const STANDING: [ItemKind; 3] = [ItemKind::Alias, ItemKind::Const, ItemKind::Macro];
    const DECLARING: [ItemKind; 3] = [ItemKind::Struct, ItemKind::Enum, ItemKind::Trait];

    pub(crate) fn sites(&self) -> &BTreeMap<Member<'file>, Site<'file>> {
        &self.sites
    }

    pub(crate) fn held(&self) -> Held<'file> {
        let shipped = self.model.holdings().filter(|placed| Shipped::admits(placed.entity()).is_some());
        let links = shipped.filter_map(|placed| {
            let holding: &Holding = placed.entity();
            let holder = *self.declarations.get(&holding.declares()?)?;
            Some(self.linked(holder, [AsRef::<TypeRef>::as_ref(holding)].into_iter().collect()))
        });
        Held(links.flatten().collect())
    }

    pub(crate) fn implemented(&self) -> Vec<Implemented<'file>> {
        let written = self.model.implementations().filter(|placed| placed.entity().origin() == Origin::Written);
        written
            .filter(|placed| Shipped::admits(placed.entity()).is_some())
            .filter_map(|placed| {
                let implementation: &Implementation = placed.entity();
                let interface = *self.declarations.get(&implementation.implemented()?.crate_item()?)?;
                let subject = implementation.subject_type().named().map(AsRef::<Resolved>::as_ref);
                let holder = subject.and_then(Resolved::crate_item).and_then(|item| self.declarations.get(&item));
                Some(holder.copied().map_or_else(
                    || {
                        Implemented::Outside(Outsider::new(
                            implementation.subject_type().brief(),
                            placed.site(),
                            interface,
                        ))
                    },
                    |found| Implemented::Inside(Link::new(found, interface)),
                ))
            })
            .collect()
    }

    pub(crate) fn used(&self) -> Vec<Link<'file>> {
        let shipped = self.model.functions().filter(|placed| Shipped::admits(placed.entity()).is_some());
        let signed = shipped.filter_map(|placed| {
            let function: &Function = placed.entity();
            let holder = *self.declarations.get(&AsRef::<Seat>::as_ref(function).owner()?)?;
            Some(self.linked(holder, function.signature().written().collect()))
        });
        let bodies = self.references.targets().iter().filter_map(|(owner, used)| {
            let holder = *self.declarations.get(owner)?;
            Some(self.linked(holder, Reached(used.clone())))
        });
        signed.chain(bodies).flatten().collect()
    }

    pub(crate) fn declared(&self) -> impl Iterator<Item = Member<'file>> + '_ {
        self.declarations.values().copied()
    }

    fn linked(&self, holder: Member<'file>, reached: Reached) -> Vec<Link<'file>> {
        let targets = self.references.targets();
        let standing = |item: ItemId| {
            let kind = self.model.declared(item).map(code_model::Item::kind);
            kind.filter(|found| Self::STANDING.contains(found)).map(|_| item)
        };
        let next = |item: ItemId| {
            let krate = self.crates.get(&item);
            let used = standing(item).and_then(|found| targets.get(&found)).into_iter().flatten().copied();
            used.filter(move |target| self.crates.get(target) == krate)
        };
        let ends = Traversal::trees(reached.0, &next).into_iter().flatten();
        ends.filter_map(|end| self.declarations.get(&end)).map(|member| Link::new(holder, *member)).collect()
    }

    fn member(model: &'file Model, placed: Placed<'file, &'file ItemDeclaration>) -> Option<Declared<'file>> {
        let declaration = placed.entity();
        Shipped::admits(declaration)?;
        let item = model.declared(declaration.declared()).filter(|found| Self::DECLARING.contains(&found.kind()))?;
        let member = Member::declared(placed.krate(), item.as_ref()).within(placed.module())?;
        Some(Declared { item: declaration.declared(), member, site: placed.site() })
    }
}

#[derive(Debug)]
pub(crate) enum Implemented<'file> {
    Inside(Link<'file>),
    Outside(Outsider<'file, 'file>),
}

impl<'file> Implemented<'file> {
    pub(crate) fn link(&self) -> Option<Link<'file>> {
        match self {
            Implemented::Inside(link) => Some(*link),
            Implemented::Outside(_) => None,
        }
    }

    pub(crate) fn outsider(&self) -> Option<Outsider<'file, 'file>> {
        match self {
            Implemented::Inside(_) => None,
            Implemented::Outside(outsider) => Some(outsider.clone()),
        }
    }
}

#[derive(Debug)]
struct Shipped;

impl Shipped {
    fn admits(entity: &impl AsRef<Region>) -> Option<()> {
        Some(entity.as_ref().reach()).filter(|reach| *reach == Reach::Shipped).map(|_| ())
    }
}

#[derive(Debug)]
pub(crate) struct Held<'file>(Vec<Link<'file>>);

impl<'file> IntoIterator for Held<'file> {
    type Item = Link<'file>;
    type IntoIter = std::vec::IntoIter<Link<'file>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
