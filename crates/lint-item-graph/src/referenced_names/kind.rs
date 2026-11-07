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
    pub fn of_item(item: &Item) -> Self {
        match item {
            Item::Struct(_) | Item::Enum(_) | Item::Trait(_) | Item::Union(_) => Self::Type,
            Item::Type(_) => Self::Alias,
            Item::Const(_) => Self::StandIn,
            Item::Macro(call) => MacroItems::defined_by(call).map_or(Self::Plain, |_| Self::StandIn),
            _ => Self::Plain,
        }
    }

    pub fn declaring(self, owner: Option<Name>) -> ReferencedNames {
        match self {
            Self::Type => ReferencedNames { types: owner.into_iter().collect(), ..ReferencedNames::default() },
            Self::StandIn => ReferencedNames { stand_ins: owner.into_iter().collect(), ..ReferencedNames::default() },
            Self::Alias => ReferencedNames {
                stand_ins: owner.iter().cloned().collect(),
                types: owner.into_iter().collect(),
                ..ReferencedNames::default()
            },
            Self::Plain => ReferencedNames::default(),
        }
    }
}
