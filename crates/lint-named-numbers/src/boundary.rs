use proc_macro2::Span;
use syn::{LitFloat, LitInt};

pub trait Numeral {
    fn digits(&self) -> &str;

    fn spot(&self) -> Span;
}

impl Numeral for LitInt {
    fn digits(&self) -> &str {
        self.base10_digits()
    }

    fn spot(&self) -> Span {
        self.span()
    }
}

impl Numeral for LitFloat {
    fn digits(&self) -> &str {
        self.base10_digits()
    }

    fn spot(&self) -> Span {
        self.span()
    }
}
