mod boundary;

use syntax::{Ident, LitStr, Variant};

use boundary::Snake;

pub struct Row {
    variant: Ident,
    name: LitStr,
}

impl Row {
    const ATTRIBUTE: &'static str = "name";

    pub fn declared(variant: &Variant) -> syntax::Result<Vec<Row>> {
        let named = variant
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(|attribute| attribute.parse_args().map(|name| Row { variant: variant.ident.clone(), name }))
            .collect::<syntax::Result<Vec<Row>>>()?;
        Ok(Row::defaulted(named, &variant.ident))
    }

    fn defaulted(named: Vec<Row>, variant: &Ident) -> Vec<Row> {
        match named.first() {
            None => vec![Row::spelled(variant)],
            Some(_declared) => named,
        }
    }

    fn spelled(variant: &Ident) -> Row {
        Row { variant: variant.clone(), name: Snake::spelled(variant) }
    }

    pub fn variant(&self) -> &Ident {
        &self.variant
    }

    pub fn name(&self) -> &LitStr {
        &self.name
    }
}
