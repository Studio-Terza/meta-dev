use syn::{Item, Stmt, Type};

use crate::uses::{Import, Place, Uses};
use lint_layout::Name;

#[derive(Clone, Debug, Default)]
pub(crate) struct Scope {
    uses: Uses,
    self_type: Option<Name>,
}

impl Scope {
    const SELF: &'static str = "Self";

    pub(crate) fn within<'item>(&self, items: impl Iterator<Item = &'item Item>) -> Scope {
        self.clone().using(items.map(Self::item_uses).fold(Uses::default(), Uses::merged))
    }

    pub(crate) fn within_statements(&self, statements: &[Stmt]) -> Scope {
        self.within(statements.iter().filter_map(Self::statement_item))
    }

    pub(crate) fn implementing(&self, self_type: &Type) -> Scope {
        Scope { self_type: self.type_named(self_type), ..self.clone() }
    }

    pub(crate) fn named(&self, written: &Name) -> Option<Name> {
        Some(written)
            .filter(|name| **name != Self::SELF)
            .map(|name| self.uses.import(name).map_or(name, Import::name))
            .or(self.self_type.as_ref())
            .cloned()
    }

    pub(crate) fn places<'scope>(&'scope self, prefix: &'scope [Name]) -> Vec<Place<'scope>> {
        prefix.split_first().map_or_else(Vec::new, |(first, rest)| self.places_of(first, rest.last()))
    }

    pub(crate) fn type_named(&self, found: &Type) -> Option<Name> {
        match found {
            Type::Reference(reference) => self.type_named(&reference.elem),
            Type::Paren(inner) => self.type_named(&inner.elem),
            Type::Path(_) => Name::of_type(found).and_then(|written| self.named(&written)),
            _ => None,
        }
    }

    fn places_of<'scope>(&'scope self, first: &'scope Name, last: Option<&'scope Name>) -> Vec<Place<'scope>> {
        match last {
            None => self.uses.sources(first),
            Some(last) => Vec::from([self.uses.qualified(first, last)]),
        }
    }

    fn using(self, uses: Uses) -> Scope {
        Scope { uses: self.uses.merged(uses), ..self }
    }

    fn statement_item(statement: &Stmt) -> Option<&Item> {
        if let Stmt::Item(item) = statement { Some(item) } else { None }
    }

    fn item_uses(item: &Item) -> Uses {
        match item {
            Item::Use(used) => Uses::of_tree(&used.tree),
            _ => Uses::default(),
        }
    }
}
