use syn::punctuated::Punctuated;
use syn::{Attribute, Path, Token};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct Derived(Vec<Name>);

impl Derived {
    const DERIVE: &'static str = "derive";
    const WRAPPING: [Word<'static>; 1] = words!["Newtype"];
    const GENERATING: [Word<'static>; 1] = words!["Arbitrary"];

    pub fn lacks_generator(&self) -> bool {
        self.0.iter().any(|name| name.matches::<Whole>(&Self::WRAPPING))
            && !self.0.iter().any(|name| name.matches::<Whole>(&Self::GENERATING))
    }
}

impl From<&[Attribute]> for Derived {
    fn from(attributes: &[Attribute]) -> Derived {
        Derived(
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
