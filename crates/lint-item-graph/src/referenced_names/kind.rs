use syn::Item;

use lint_layout::Name;

use super::ReferencedNames;
use super::macro_items::MacroItems;

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Type,
    StandIn,
    Alias,
    Plain,
}

impl Kind {
    pub fn of_item(item: &Item) -> Kind {
        match item {
            Item::Struct(_) | Item::Enum(_) | Item::Trait(_) | Item::Union(_) => Kind::Type,
            Item::Type(_) => Kind::Alias,
            Item::Const(_) => Kind::StandIn,
            Item::Macro(call) => MacroItems::defined_by(call).map_or(Kind::Plain, |_| Kind::StandIn),
            _ => Kind::Plain,
        }
    }

    pub fn declaring(self, owner: Option<Name>) -> ReferencedNames {
        let (stand_in, typed) = match self {
            Kind::Type => (None, owner),
            Kind::StandIn => (owner, None),
            Kind::Alias => (owner.clone(), owner),
            Kind::Plain => (None, None),
        };
        ReferencedNames {
            stand_ins: stand_in.into_iter().collect(),
            types: typed.into_iter().collect(),
            ..ReferencedNames::default()
        }
    }
}
