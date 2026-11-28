use quote::format_ident;
use syn::{Ident, ReturnType};

use named::Slot;

mod named;

pub(crate) enum Returned {
    Plain,
    Acquired(ReturnType),
}

impl Returned {
    pub(crate) fn parsed(output: &ReturnType) -> Returned {
        match output {
            ReturnType::Default => Returned::Plain,
            ReturnType::Type(arrow, tree) => Slot::unwrapped(*tree.clone())
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
}
