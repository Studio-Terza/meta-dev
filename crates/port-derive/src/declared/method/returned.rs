use quote::format_ident;
use syn::{Ident, ReturnType, Type};

use named::{Named, Slot};

mod named;

pub(crate) enum Returned {
    Plain,
    Acquired(ReturnType),
}

impl Returned {
    pub(crate) fn parsed(output: &ReturnType) -> Returned {
        match output {
            ReturnType::Default => Returned::Plain,
            ReturnType::Type(arrow, tree) => Self::unheld(tree)
                .map_or(Returned::Plain, |filler| Returned::Acquired(ReturnType::Type(*arrow, Box::new(filler)))),
        }
    }

    pub(crate) fn filler_output<'method>(&'method self, declared: &'method ReturnType) -> &'method ReturnType {
        match self {
            Returned::Plain => declared,
            Returned::Acquired(filler) => filler,
        }
    }

    pub(crate) fn counting(&self) -> Ident {
        match self {
            Returned::Plain => format_ident!("counted_call"),
            Returned::Acquired(_) => format_ident!("counted_acquire"),
        }
    }

    fn unheld(tree: &Type) -> Option<Type> {
        Named::read(tree).and_then(Named::resource).cloned().or_else(|| Slot::unwrapped(tree.clone()))
    }
}
