mod finding;

use std::collections::BTreeMap;

use graph::{Ground, Harness, Implementation, ItemId, ItemKind, ItemPath, Model, Placed, Region, Resolved, Spelling};
use lint_core::{Site, Violation};
use lint_layout::{CamelWord, Name, Whole, Word, words};

use finding::Finding;

#[derive(Debug)]
struct Double<'model> {
    name: &'model Name,
    site: Site<'model>,
}

#[derive(Debug)]
struct Implemented<'model> {
    double: ItemId,
    placed: Placed<'model, &'model Implementation>,
    kind: Kind,
}

#[derive(Debug)]
pub(crate) struct Doubles<'model> {
    declared: BTreeMap<ItemId, Double<'model>>,
    ported: BTreeMap<ItemId, Ground>,
}

impl<'model> From<&'model Model> for Doubles<'model> {
    fn from(model: &'model Model) -> Doubles<'model> {
        let declared = model
            .items()
            .filter(|placed| placed.harness() == Harness::TestTarget)
            .filter_map(|placed| {
                let id = placed.entity().declared();
                let item = model.declared(id).filter(|item| Self::TYPES.contains(&item.kind()))?;
                let name: &Name = item.as_ref();
                name.matches::<CamelWord>(&Self::DOUBLES).map(|_| (id, Double { name, site: placed.site() }))
            })
            .collect();
        let ported = model
            .items()
            .map(|placed| (placed.entity().declared(), AsRef::<Region>::as_ref(placed.entity()).ground()))
            .collect();
        Doubles { declared, ported }
    }
}

impl<'model> Doubles<'model> {
    const TYPES: [ItemKind; 3] = [ItemKind::Struct, ItemKind::Enum, ItemKind::Union];
    const DOUBLES: [Word<'static>; 5] = words!["fake", "stub", "mock", "spy", "dummy"];
    const PORTS: Word<'static> = Word::new("ports");
    const STD: [Word<'static>; 3] = words!["std", "core", "alloc"];

    pub(crate) fn violations(&self, model: &'model Model) -> Vec<Violation<'model>> {
        let implemented: Vec<Implemented<'model>> = model
            .implementations()
            .filter(|placed| placed.entity().written().is_some())
            .filter_map(|placed| {
                let implementation = placed.entity();
                let target: &Resolved = implementation.subject_type().named()?.as_ref();
                let id = target.crate_item().filter(|id| self.declared.contains_key(id))?;
                implementation.implemented().map(|_| Implemented {
                    double: id,
                    placed,
                    kind: self.kind(implementation),
                })
            })
            .collect();
        let unported = self
            .declared
            .iter()
            .filter(|(id, _)| {
                implemented.iter().filter(|found| found.double == **id).all(|found| found.kind != Kind::Port)
            })
            .map(|(_, double)| Violation::new(double.site, Finding::Unported(double.name)));
        let beside = implemented.iter().filter(|found| found.kind == Kind::Other).filter_map(|found| {
            let placed = found.placed;
            let written = Self::written(placed.entity())?;
            self.declared
                .get(&found.double)
                .map(|double| Violation::new(placed.site(), Finding::Beside(double.name, written.clone())))
        });
        let mut violations: Vec<Violation<'model>> = unported.chain(beside).collect();
        violations.sort_by(|left, right| left.site().cmp(right.site()));
        violations
    }

    fn kind(&self, implementation: &Implementation) -> Kind {
        let resolved = implementation.implemented();
        let standard = resolved.and_then(Resolved::std).map(|_| Kind::Std);
        let declared = || resolved.and_then(Resolved::crate_item).map(|id| self.grounded(id));
        let external = || resolved.and_then(Resolved::external).map(|found| Self::pathed(found.path()));
        let written = || Self::written(implementation).map(Self::pathed);
        standard.or_else(declared).or_else(external).or_else(written).unwrap_or(Kind::Other)
    }

    fn grounded(&self, id: ItemId) -> Kind {
        self.ported.get(&id).filter(|ground| **ground == Ground::Ports).map_or(Kind::Other, |_| Kind::Port)
    }

    fn written(implementation: &Implementation) -> Option<&ItemPath> {
        AsRef::<Spelling>::as_ref(implementation.interface()?).written()
    }

    fn pathed(path: &ItemPath) -> Kind {
        let segments: &[Name] = path.as_ref();
        let modules = segments.split_last().map_or(&[][..], |(_, modules)| modules);
        let first = modules.first();
        let named = modules.iter().find(|module| **module == Name::from(Self::PORTS)).map(|_| Kind::Port);
        let suffixed =
            first.filter(|module| module.words().last() == Some(&Name::from(Self::PORTS))).map(|_| Kind::Port);
        let standard = first.filter(|module| module.matches::<Whole>(&Self::STD).is_some()).map(|_| Kind::Std);
        named.or(suffixed).or(standard).unwrap_or(Kind::Other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Port,
    Std,
    Other,
}
