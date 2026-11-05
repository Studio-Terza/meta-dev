use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

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

mod sealed {
    use super::{FromStr, Unnamed, VariantArray, fmt};

    pub trait Sealed {}

    impl<Of: VariantArray + fmt::Display + FromStr<Err = Unnamed<Of>>> Sealed for Of {}
}

pub trait Listed: sealed::Sealed + VariantArray + fmt::Display {}

impl<Of: sealed::Sealed + VariantArray + fmt::Display> Listed for Of {}

impl<Of: Listed> Unnamed<Of> {
    #[must_use]
    pub fn new(text: &str) -> Unnamed<Of> {
        assert!(
            Of::VARIANTS.iter().all(|variant| variant.to_string() != text),
            "`{text}` names a variant, so it cannot be an Unnamed"
        );
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
