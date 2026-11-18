mod boundary;

use syntax::{Attribute, Ident, LitStr, Variant};

use super::case::Case;
use boundary::Spelled;

pub struct Row {
    variant: Ident,
    name: LitStr,
}

impl Row {
    const ATTRIBUTE: &'static str = "name";

    pub fn declared(variant: &Variant, case: Case) -> syntax::Result<Vec<Row>> {
        let spelled = Row::spelled(&variant.ident, case);
        let named = variant
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(|attribute| Row::named(attribute, &spelled))
            .collect::<syntax::Result<Vec<Row>>>()?;
        Ok(Row::defaulted(named, spelled))
    }

    fn named(attribute: &Attribute, spelled: &Row) -> syntax::Result<Row> {
        let name: LitStr = attribute.parse_args()?;
        let span = name.span();
        (name.value() != spelled.name.value())
            .then_some(Row { variant: spelled.variant.clone(), name })
            .ok_or_else(|| syntax::Error::new(span, "this #[name] is the variant's default spelling; remove it"))
    }

    fn defaulted(named: Vec<Row>, spelled: Row) -> Vec<Row> {
        match named.first() {
            None => vec![spelled],
            Some(_declared) => named,
        }
    }

    fn spelled(variant: &Ident, case: Case) -> Row {
        Row { variant: variant.clone(), name: Spelled::spelled(variant, case) }
    }

    pub fn variant(&self) -> &Ident {
        &self.variant
    }

    pub fn name(&self) -> &LitStr {
        &self.name
    }
}
