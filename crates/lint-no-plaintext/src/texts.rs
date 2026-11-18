use std::collections::BTreeMap;
use std::iter;

use graph::{
    Held, Holding, ItemId, ItemPath, Model, Named, Placed, Primitive, Resolved, Sited, Spelling, TypeRef, Unresolved,
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
        let rounds = iter::successors(Some(Texts::default()), |settled| {
            let widened = settled.widened(&wrappers);
            Some(widened).filter(|next| next != settled)
        });
        rounds.take(wrappers.len() + 1).last().unwrap_or_default()
    }

    pub(crate) fn content(&self, ty: &TypeRef) -> Option<Content> {
        let string: ItemPath = Self::STRING.into_iter().collect();
        ty.uses().into_iter().filter_map(|named| self.leaf(&string, named.as_ref())).min()
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
pub(crate) struct Spots;

impl Spots {
    pub(crate) fn first(ty: &TypeRef) -> Option<Position> {
        ty.uses().into_iter().find_map(Spots::spotted)
    }

    fn spotted(named: &Named<Sited>) -> Option<Position> {
        let sited: &Sited = named.as_ref();
        AsRef::<Spelling>::as_ref(sited).position()
    }
}
