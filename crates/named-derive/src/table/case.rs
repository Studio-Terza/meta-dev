use syntax::{Attribute, Ident};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Snake,
    Kebab,
}

impl Case {
    const ATTRIBUTE: &'static str = "named";
    const SPELLED: [(&'static str, Case); 1] = [("kebab", Case::Kebab)];

    pub fn declared(attributes: &[Attribute]) -> syntax::Result<Case> {
        attributes
            .iter()
            .find(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map_or(Ok(Case::Snake), Case::read)
    }

    fn read(attribute: &Attribute) -> syntax::Result<Case> {
        let word: Ident = attribute.parse_args()?;
        Self::SPELLED
            .into_iter()
            .find(|(spelled, _)| word == *spelled)
            .map(|(_, case)| case)
            .ok_or_else(|| syntax::Error::new(word.span(), "expected #[named(kebab)]"))
    }
}
