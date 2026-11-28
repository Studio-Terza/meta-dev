use proc_macro2::Ident;
use syn::{ExprMethodCall, ItemType, Macro, Path};

use crate::finding::Found;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::style;

#[derive(Debug, Default)]
pub(crate) struct HiddenTypes {
    found: Vec<Found>,
}

impl HiddenTypes {
    const ERASING: [&'static str; 6] = ["Any", "TypeId", "type_id", "downcast", "downcast_ref", "downcast_mut"];

    fn erased(&mut self, name: &Ident) {
        let erasing = HiddenTypes::ERASING.iter().any(|erasing| name == erasing);
        self.found.extend(erasing.then(|| {
            Found::spanned(
                name.span(),
                format_args!("{name} erases a type"),
                "take a generic or name an enum of the known types",
            )
        }));
    }
}

impl Hooks for HiddenTypes {
    fn path(&mut self, path: &Path) -> Descent {
        path.segments.iter().for_each(|segment| self.erased(&segment.ident));
        Descent::Into
    }

    fn expr_method_call(&mut self, call: &ExprMethodCall) -> Descent {
        self.erased(&call.method);
        Descent::Into
    }

    fn item_type(&mut self, alias: &ItemType) -> Descent {
        let name = &alias.ident;
        self.found.extend([Found::spanned(
            name.span(),
            format_args!("type {name} renames a type"),
            "name the type itself, or make the alias a newtype",
        )]);
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }
}

style!(
    HiddenTypes,
    Every,
    "no hidden types: a type stays in the compiler's view, never erased to Any, renamed by an alias or carried as untyped data"
);
