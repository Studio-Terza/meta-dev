use syn::{Attribute, ItemTrait};

use lint_layout::{Name, Word, words};

#[derive(Clone, Copy, Debug)]
pub(crate) struct PortMacro;

impl PortMacro {
    const PORT: [Word<'static>; 1] = words!["port"];
    const QUALIFIED: [Word<'static>; 2] = words!["port_core", "port"];
    pub(crate) const FILLER: Word<'static> = Word::new("Filler");

    pub(crate) fn filler(item: &ItemTrait) -> Option<Name> {
        Self::marked(item).map(|found| Name::from(&found.ident).suffixed(Self::FILLER))
    }

    pub(crate) fn marked(item: &ItemTrait) -> Option<&ItemTrait> {
        item.attrs.iter().find_map(Self::marking).map(|_| item)
    }

    fn marking(attribute: &Attribute) -> Option<&Attribute> {
        let written: Vec<Name> = attribute.path().segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        [Self::PORT.as_slice(), Self::QUALIFIED.as_slice()]
            .into_iter()
            .map(|mark| mark.iter().copied().map(Name::from).collect::<Vec<Name>>())
            .find(|mark| *mark == written)
            .map(|_| attribute)
    }
}
