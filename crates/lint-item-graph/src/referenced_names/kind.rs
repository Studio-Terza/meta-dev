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
        match self {
            Kind::Type => ReferencedNames { types: owner.into_iter().collect(), ..ReferencedNames::default() },
            Kind::StandIn => ReferencedNames { stand_ins: owner.into_iter().collect(), ..ReferencedNames::default() },
            Kind::Alias => ReferencedNames {
                stand_ins: owner.iter().cloned().collect(),
                types: owner.into_iter().collect(),
                ..ReferencedNames::default()
            },
            Kind::Plain => ReferencedNames::default(),
        }
    }
}
