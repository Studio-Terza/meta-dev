use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

use strum::VariantArray;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unnamed<Of> {
    text: String,
    of: PhantomData<Of>,
}

impl<Of> Unnamed<Of> {
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

impl<Of: VariantArray + fmt::Display + fmt::Debug> Error for Unnamed<Of> {}
