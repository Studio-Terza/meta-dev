use syn::{Item, ItemMacro, Stmt};

use lint_layout::Name;
use macro_facts::Macros;

#[derive(Debug)]
pub struct MacroItems;

impl MacroItems {
    const DEFINITION: &str = "macro_rules";

    pub fn declared_by(call: &ItemMacro) -> Vec<Item> {
        call.mac.path.get_ident().filter(|ident| *ident == Self::DEFINITION).map_or_else(
            || Macros::statements(&call.mac).into_iter().filter_map(Self::declared_item).collect(),
            |_| Vec::new(),
        )
    }

    pub fn defined_by(call: &ItemMacro) -> Option<Name> {
        call.ident.as_ref().filter(|_| call.mac.path.is_ident(Self::DEFINITION)).map(Name::from)
    }

    fn declared_item(statement: Stmt) -> Option<Item> {
        match statement {
            Stmt::Item(item) => Some(item),
            Stmt::Local(_) | Stmt::Expr(..) | Stmt::Macro(_) => None,
        }
    }
}
