use syn::{Item, ItemMacro, Stmt};

use lint_core::Macros;

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

    fn declared_item(statement: Stmt) -> Option<Item> {
        match statement {
            Stmt::Item(item) => Some(item),
            Stmt::Local(_) | Stmt::Expr(..) | Stmt::Macro(_) => None,
        }
    }
}
