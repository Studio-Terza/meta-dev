use syn::Attribute;

use lint_core::AttributeMacros;
use lint_layout::Whole;
use lint_layout::{Name, Word, words};

#[derive(Debug)]
pub(crate) struct Derives;

impl Derives {
    const SERDE: [Word<'static>; 2] = words!["Serialize", "Deserialize"];

    pub(crate) fn named(attributes: &[Attribute]) -> Vec<Name> {
        attributes
            .iter()
            .flat_map(|attribute| AttributeMacros::expanded(&attribute.meta))
            .filter_map(|path| path.segments.last().map(|last| Name::from(&last.ident)))
            .filter(|name| name.matches::<Whole>(&Self::SERDE).is_some())
            .collect()
    }
}
