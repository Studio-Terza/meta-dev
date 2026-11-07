use syn::punctuated::Punctuated;
use syn::{Attribute, Path, Token};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct Derived(Vec<Name>);

impl Derived {
    const DERIVE: &'static str = "derive";
    const WRAPPING: [Word<'static>; 1] = words!["Newtype"];
    const GENERATING: [Word<'static>; 1] = words!["Arbitrary"];

    pub fn lacks_generator(&self) -> Option<&Name> {
        match self.0.iter().find_map(|name| name.matches::<Whole>(&Self::GENERATING)) {
            Some(_) => None,
            None => self.0.iter().find(|name| name.matches::<Whole>(&Self::WRAPPING).is_some()),
        }
    }
}

impl From<&[Attribute]> for Derived {
    fn from(attributes: &[Attribute]) -> Self {
        Self(
            attributes
                .iter()
                .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
                .filter_map(|attribute| attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
                .flatten()
                .filter_map(|path| path.segments.last().map(|last| Name::from(&last.ident)))
                .collect(),
        )
    }
}
