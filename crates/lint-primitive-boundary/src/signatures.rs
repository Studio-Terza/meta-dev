mod walk;

use std::path::Path;

use code_model::{Function, Held, Holding, ItemId, Model, Named, Owner, Region, Resolved, Seat, Sited, TypeRef, Zone};
use lint_layout::{Name, Whole, Word, words};

use crate::judged::{Judged, Scope};
use crate::legacy::Legacy;
use crate::lexicon::Lexicon;
use crate::lookups::Lookups;
use walk::TypeWalk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Wrapped {
    Bare,
    Composite,
}

impl From<Option<&Named<Sited>>> for Wrapped {
    fn from(shared: Option<&Named<Sited>>) -> Wrapped {
        match shared {
            Some(_) => Wrapped::Bare,
            None => Wrapped::Composite,
        }
    }
}

impl Wrapped {
    fn wrapping(lexicon: &Lexicon, ty: &TypeRef) -> Wrapped {
        match ty.dereferenced().named() {
            Some(named) => Wrapped::of_named(lexicon, named),
            None => Wrapped::Composite,
        }
    }

    fn of_named(lexicon: &Lexicon, named: &Named<Sited>) -> Wrapped {
        match named.arguments().first() {
            Some(_) => Wrapped::from(lexicon.shared_str(named)),
            None => Wrapped::Bare,
        }
    }

    fn kept(self, holding: &Holding) -> Option<&Holding> {
        match self {
            Wrapped::Bare => None,
            Wrapped::Composite => Some(holding),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Holdings;

impl Holdings {
    fn legacy(held: Held) -> Legacy {
        match held {
            Held::StructField | Held::EnumField | Held::UnionField | Held::StructParameter => Legacy::Eligible,
            Held::NewtypeField
            | Held::EnumParameter
            | Held::UnionParameter
            | Held::Const
            | Held::Static
            | Held::Alias
            | Held::AssociatedType
            | Held::AssociatedConst => Legacy::Ineligible,
        }
    }

    fn kept<'holding>(lexicon: &Lexicon, holding: &'holding Holding) -> Option<&'holding Holding> {
        match holding.held() {
            Held::NewtypeField => Wrapped::wrapping(lexicon, holding.as_ref()).kept(holding),
            Held::StructField
            | Held::EnumField
            | Held::UnionField
            | Held::StructParameter
            | Held::EnumParameter
            | Held::UnionParameter
            | Held::Const
            | Held::Static
            | Held::Alias
            | Held::AssociatedType
            | Held::AssociatedConst => Some(holding),
        }
    }
}

#[derive(Debug)]
pub struct Signatures<'read, 'model> {
    lexicon: &'read Lexicon,
    lookups: &'read Lookups<'model>,
}

impl<'read, 'model> Signatures<'read, 'model> {
    const EDGES: [Word<'static>; 3] = words!["new", "try_new", "into_inner"];

    pub fn new(lexicon: &'read Lexicon, lookups: &'read Lookups<'model>) -> Signatures<'read, 'model> {
        Signatures { lexicon, lookups }
    }

    pub fn judged(&self, model: &'model Model) -> Vec<Judged<'model>> {
        let functions = model
            .functions()
            .everywhere()
            .filter(|placed| placed.entity().owner() != Owner::TraitImpl)
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).zone() == Zone::Open)
            .flat_map(|placed| self.function_judged(placed.entity(), placed.site().path()));
        let holdings = model
            .holdings()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).zone() == Zone::Open)
            .filter_map(|placed| Holdings::kept(self.lexicon, placed.entity()).map(|_| placed))
            .flat_map(|placed| self.holding_judged(placed.entity(), placed.site().path()));
        functions.chain(holdings).collect()
    }

    fn function_judged(&self, function: &'model Function, path: &'model Path) -> Vec<Judged<'model>> {
        let region: Region = *AsRef::<Region>::as_ref(function);
        let eligible = Legacy::of_within(region.within()).both(Legacy::unless(self.edge(function, region).as_ref()));
        let candidates = TypeWalk::walked(self.lexicon, self.lookups, path, function.signature().written());
        Scope::new(path, region, eligible, function.as_ref()).judged(candidates)
    }

    fn holding_judged(&self, holding: &'model Holding, path: &'model Path) -> Vec<Judged<'model>> {
        let region: Region = *AsRef::<Region>::as_ref(holding);
        let newtype = self.lookups.newtype(holding.declares(), region.reach());
        let eligible = Legacy::of_within(region.within())
            .both(Holdings::legacy(holding.held()))
            .both(Legacy::unless(newtype.as_ref()));
        let candidates =
            TypeWalk::walked(self.lexicon, self.lookups, path, [AsRef::<TypeRef>::as_ref(holding)].into_iter());
        Scope::new(path, region, eligible, holding.as_ref()).judged(candidates)
    }

    fn edge(&self, function: &Function, region: Region) -> Option<ItemId> {
        let inherent = [function.owner()].into_iter().find(|owner| *owner == Owner::Inherent);
        let inputs = function.signature().inputs();
        let alone = inputs.first().xor(inputs.get(1));
        let name: &Name = function.as_ref();
        let edge = name.matches::<Whole>(&Self::EDGES);
        let seat: &Seat = function.as_ref();
        let item =
            seat.subject().and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref).and_then(Resolved::crate_item);
        let newtype = self.lookups.newtype(item, region.reach());
        inherent.and(alone).and(edge).and(newtype)
    }
}
