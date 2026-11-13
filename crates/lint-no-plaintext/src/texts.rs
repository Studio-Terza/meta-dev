use std::collections::BTreeMap;

use graph::{
    Held, Holding, ItemId, ItemPath, Model, Named, Placed, Primitive, Resolved, Sited, Spelling, TypeRef, TypeUses,
    Unresolved,
};
use lint_layout::{Position, Word, words};

use crate::kinds::Kinds;

#[derive(Clone, Copy, Debug)]
struct Field<'ty> {
    item: ItemId,
    held: &'ty TypeRef,
}

#[derive(Debug)]
struct Wrapper {
    item: ItemId,
    held: TypeRef,
}

impl Wrapper {
    fn read(model: &Model, kinds: &Kinds<'_>) -> Vec<Wrapper> {
        let holdings: Vec<&Holding> = model.holdings().map(Placed::entity).collect();
        let newtypes = holdings
            .iter()
            .filter(|holding| holding.held() == Held::NewtypeField)
            .filter_map(|holding| Some(Field { item: holding.declares()?, held: holding.as_ref() }));
        let fielded = holdings
            .iter()
            .filter(|holding| holding.held() == Held::StructField)
            .filter_map(|holding| Some(Field { item: holding.declares()?, held: holding.as_ref() }))
            .fold(BTreeMap::new(), |mut fielded: BTreeMap<ItemId, Vec<Field<'_>>>, field| {
                fielded.entry(field.item).or_default().push(field);
                fielded
            });
        let lone = fielded.into_values().filter_map(|fields| fields.first().xor(fields.get(1)).copied());
        newtypes
            .chain(lone)
            .filter_map(|field| {
                Some(Wrapper { item: kinds.unvalidated(field.item)?, held: model.expanded(field.held) })
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Content {
    Text,
    Unresolved,
    MacroDeclared,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Texts {
    wrappers: BTreeMap<ItemId, Content>,
}

impl Texts {
    const STRING: [Word<'static>; 3] = words!["std", "string", "String"];

    pub(crate) fn settled(model: &Model, kinds: &Kinds<'_>) -> Texts {
        let wrappers = Wrapper::read(model, kinds);
        wrappers.iter().fold(Texts::default(), |settled, _| settled.widened(&wrappers))
    }

    pub(crate) fn content(&self, ty: &TypeRef) -> Option<Content> {
        let string: ItemPath = Self::STRING.into_iter().collect();
        let mut leaves = Leaves { texts: self, string, found: Vec::new() };
        ty.each_use(&mut leaves);
        leaves.found.into_iter().min()
    }

    fn widened(&self, wrappers: &[Wrapper]) -> Texts {
        Texts {
            wrappers: wrappers
                .iter()
                .filter_map(|wrapper| Some((wrapper.item, self.content(&wrapper.held)?)))
                .collect(),
        }
    }

    fn leaf(&self, string: &ItemPath, resolved: &Resolved) -> Option<Content> {
        let text = |known: &Resolved| {
            let owned = known.std().filter(|std| *std == string).map(|_| Content::Text);
            let borrowed = known.primitive().filter(|primitive| *primitive == Primitive::Str).map(|_| Content::Text);
            let wrapped = known.crate_item().and_then(|item| self.wrappers.get(&item)).copied();
            owned.or(borrowed).or(wrapped)
        };
        resolved.unresolved().map_or_else(|| text(resolved), |cause| Some(Self::unresolved(cause)))
    }

    fn unresolved(cause: Unresolved) -> Content {
        match cause {
            Unresolved::Unbound | Unresolved::Unread => Content::Unresolved,
            Unresolved::PossiblyMacroDeclared => Content::MacroDeclared,
        }
    }
}

#[derive(Debug)]
struct Leaves<'texts> {
    texts: &'texts Texts,
    string: ItemPath,
    found: Vec<Content>,
}

impl TypeUses for Leaves<'_> {
    fn named(&mut self, named: &Named<Sited>) {
        self.found.extend(self.texts.leaf(&self.string, named.as_ref()));
        named.each_inner_use(self);
    }
}

#[derive(Debug, Default)]
pub(crate) struct Spots {
    found: Vec<Position>,
}

impl Spots {
    pub(crate) fn first(ty: &TypeRef) -> Option<Position> {
        let mut spots = Spots::default();
        ty.each_use(&mut spots);
        spots.found.into_iter().next()
    }
}

impl TypeUses for Spots {
    fn named(&mut self, named: &Named<Sited>) {
        let sited: &Sited = named.as_ref();
        self.found.extend(AsRef::<Spelling>::as_ref(sited).position());
        named.each_inner_use(self);
    }
}
