use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

use strum::VariantArray;

pub struct Unnamed<Of> {
    text: String,
    of: PhantomData<fn() -> Of>,
}

impl<Of> Clone for Unnamed<Of> {
    fn clone(&self) -> Unnamed<Of> {
        Unnamed { text: self.text.clone(), of: PhantomData }
    }
}

impl<Of> fmt::Debug for Unnamed<Of> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Unnamed").field("text", &self.text).finish()
    }
}

impl<Of> PartialEq for Unnamed<Of> {
    fn eq(&self, other: &Unnamed<Of>) -> bool {
        self.text == other.text
    }
}

impl<Of> Eq for Unnamed<Of> {}

pub trait Listed: VariantArray + fmt::Display {}

impl<Of: Listed> Unnamed<Of> {
    #[must_use]
    pub fn new(text: &str) -> Unnamed<Of> {
        Unnamed { text: text.to_owned(), of: PhantomData }
    }
}

impl<Of: VariantArray + fmt::Display> fmt::Display for Unnamed<Of> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = Of::VARIANTS.iter().map(ToString::to_string).collect();
        write!(formatter, "`{}` is not one of: {}", self.text, names.join(", "))
    }
}

impl<Of: VariantArray + fmt::Display> Error for Unnamed<Of> {}
