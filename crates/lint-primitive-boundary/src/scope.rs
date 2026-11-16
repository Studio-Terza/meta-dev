use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use graph::{
    Call, Function, Held, Holding, Implementation, ItemId, ItemPath, Model, Owner, Placed, Region, Resolved, Seat,
    Spelling, TypeRef, Within,
};
use lint_layout::{Name, Whole, Word, words};

use crate::lexicon::Lexicon;

#[derive(Debug)]
struct AliasTarget<'model> {
    item: ItemId,
    path: &'model Path,
    target: &'model TypeRef,
}

impl<'model> AliasTarget<'model> {
    fn declared(placed: Placed<'model, &'model Holding>) -> Option<AliasTarget<'model>> {
        let holding = placed.entity();
        let alias = [holding.held()].into_iter().find(|held| *held == Held::Alias);
        alias.and(holding.declares()).map(|item| AliasTarget {
            item,
            path: placed.site().path(),
            target: holding.as_ref(),
        })
    }

    fn local(&self, file: &Path) -> Option<&'model TypeRef> {
        [self.path].into_iter().find(|path| *path == file).map(|_| self.target)
    }
}

#[derive(Debug)]
struct Door<'model> {
    path: &'model Path,
    subject: &'model Resolved,
}

impl<'model> Door<'model> {
    fn opened(placed: Placed<'model, &'model Implementation>) -> Option<Door<'model>> {
        let implementation = placed.entity();
        let open = Scope::open(*AsRef::<Region>::as_ref(implementation));
        let subject = implementation.subject_type().named().map(AsRef::<Resolved>::as_ref);
        let written = implementation.interface().and_then(|interface| AsRef::<Spelling>::as_ref(interface).written());
        let named = written.and_then(ItemPath::last).and_then(|word| word.matches::<Whole>(&Lexicon::DOOR_WORDS));
        open.and(named).and(subject).map(|opened| Door { path: placed.site().path(), subject: opened })
    }
}

#[derive(Debug)]
struct Newtypes(BTreeSet<ItemId>);

impl Newtypes {
    const EDGES: [Word<'static>; 3] = words!["new", "try_new", "into_inner"];

    fn read(model: &Model) -> Newtypes {
        Newtypes(model.holdings().filter_map(|placed| Newtypes::declared(placed.entity())).collect())
    }

    fn declared(holding: &Holding) -> Option<ItemId> {
        let newtype = [holding.held()].into_iter().find(|held| *held == Held::NewtypeField);
        newtype.and(holding.declares())
    }

    fn newtype(&self, item: Option<ItemId>) -> Option<ItemId> {
        item.filter(|id| self.0.contains(id))
    }

    fn edge(&self, function: &Function) -> Option<ItemId> {
        let name: &Name = function.as_ref();
        let edge = name.matches::<Whole>(&Self::EDGES);
        let inherent = edge.and([function.owner()].into_iter().find(|owner| *owner == Owner::Inherent));
        let inputs = function.signature().inputs();
        let alone = inherent.and(inputs.first().xor(inputs.get(1)));
        let seat: &Seat = function.as_ref();
        let item = alone
            .and(seat.subject())
            .and_then(TypeRef::named)
            .map(AsRef::<Resolved>::as_ref)
            .and_then(Resolved::crate_item);
        self.newtype(item)
    }
}

#[derive(Debug)]
pub struct Scope<'model> {
    newtypes: Newtypes,
    aliases: BTreeMap<ItemId, AliasTarget<'model>>,
    doors: BTreeMap<&'model Path, BTreeSet<&'model Resolved>>,
}

impl<'model> Scope<'model> {
    const CHECKED: [Held; 4] = [Held::StructField, Held::EnumField, Held::UnionField, Held::StructParameter];

    pub fn read(model: &'model Model) -> Scope<'model> {
        Scope {
            newtypes: Newtypes::read(model),
            aliases: model.holdings().filter_map(AliasTarget::declared).map(|alias| (alias.item, alias)).collect(),
            doors: model.implementations().filter_map(Door::opened).fold(BTreeMap::new(), |mut doors, door| {
                doors.entry(door.path).or_insert_with(BTreeSet::new).insert(door.subject);
                doors
            }),
        }
    }

    pub fn function<'function>(&self, function: &'function Function) -> Option<&'function Function> {
        let own = Scope::own(*AsRef::<Region>::as_ref(function));
        own.and_then(|_| Scope::unexempted(self.newtypes.edge(function).as_ref(), function))
    }

    pub fn holding<'holding>(&self, holding: &'holding Holding) -> Option<&'holding Holding> {
        let own = Scope::own(*AsRef::<Region>::as_ref(holding));
        let checked = own.and_then(|_| Scope::CHECKED.iter().find(|held| **held == holding.held()));
        checked.and_then(|_| Scope::unexempted(self.newtypes.newtype(holding.declares()).as_ref(), holding))
    }

    pub fn call<'call>(&self, call: &'call Call, path: &Path) -> Option<&'call Call> {
        let open = Scope::open(*AsRef::<Region>::as_ref(call));
        let seat: &Seat = call.as_ref();
        let subject = seat.subject().and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref);
        let door = subject.and_then(|named| self.doors.get(path).and_then(|subjects| subjects.get(named)));
        open.and_then(|_| Scope::unexempted(door.as_ref(), call))
    }

    pub fn local_alias(&self, resolved: &Resolved, file: &Path) -> Option<&'model TypeRef> {
        let alias = resolved.crate_item().and_then(|item| self.aliases.get(&item));
        alias.and_then(|found| found.local(file))
    }

    fn own(region: Region) -> Option<Region> {
        [region.ungrounded()].into_iter().find(|own| *own == Region::SHIPPED)
    }

    fn open(region: Region) -> Option<Within> {
        Scope::own(Region::new(region.reach(), region.zone(), Within::Own)).map(|_| region.within())
    }

    fn unexempted<Exemption, Kept>(exemption: Option<&Exemption>, kept: Kept) -> Option<Kept> {
        match exemption {
            Some(_) => None,
            None => Some(kept),
        }
    }
}
