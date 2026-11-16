use query_cache::{Origin, Table};
use syntax::parse::{Parse, ParseStream};
use syntax::{Attribute, Data, DataStruct, DeriveInput, Error, Field, Fields, Ident, LitStr, Token};

use super::{Placed, Record};

#[derive(Debug)]
struct Naming {
    table: LitStr,
}

#[derive(Clone, Copy, Debug)]
struct Marked {
    origin: Origin,
}

impl Naming {
    const TABLE: &'static str = "table";
    const UNNAMED: &'static str = "a record names its table, as in #[record(table = \"cursors\")]";
}

impl Parse for Naming {
    fn parse(input: ParseStream<'_>) -> syntax::Result<Naming> {
        let word: Ident = input.parse()?;
        Some(&word).filter(|found| *found == Self::TABLE).ok_or_else(|| Error::new(word.span(), Self::UNNAMED))?;
        input.parse::<Token![=]>()?;
        Ok(Naming { table: input.parse()? })
    }
}

impl Parse for Marked {
    fn parse(input: ParseStream<'_>) -> syntax::Result<Marked> {
        let word: Ident = input.parse()?;
        let origin = word.to_string().parse::<Origin>().map_err(|failed| Error::new(word.span(), failed))?;
        Ok(Marked { origin })
    }
}

impl Record {
    pub(super) const KEYLESS: &'static str = "a record has one field marked #[record(key)] or #[record(generated_key)]";
    const UNNAMED: &'static str = "a record is a struct with named fields";
    const ATTRIBUTE: &'static str = "record";

    pub(super) fn statements(table: &LitStr, fields: &[Placed]) -> [LitStr; 5] {
        let described = fields.iter().fold(Table::named(&table.value()), |described, field| {
            described.column(&field.name.to_string(), field.origin)
        });
        described.statements().map(|sql| LitStr::new(&sql, table.span()))
    }

    fn marking(attributes: &[Attribute]) -> Option<&Attribute> {
        attributes.iter().find(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
    }

    fn named(data: &Data) -> Option<Vec<Field>> {
        match data {
            Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => {
                Some(named.named.iter().cloned().collect())
            }
            Data::Struct(_) | Data::Enum(_) | Data::Union(_) => None,
        }
    }
}

impl Placed {
    pub(super) fn overwrite(&self, table: &LitStr) -> LitStr {
        LitStr::new(&Table::named(&table.value()).overwrite(&self.name.to_string()), table.span())
    }

    fn from_field(field: &Field) -> syntax::Result<Placed> {
        let name = field.ident.clone().ok_or_else(|| Error::new_spanned(field, Record::UNNAMED))?;
        let marked = Record::marking(&field.attrs).map(Attribute::parse_args::<Marked>).transpose()?;
        Ok(Placed { name, column: field.ty.clone(), origin: marked.map_or(Origin::Given, |found| found.origin) })
    }
}

impl Parse for Record {
    fn parse(input: ParseStream<'_>) -> syntax::Result<Record> {
        let row: DeriveInput = input.parse()?;
        let fields = Self::named(&row.data).ok_or_else(|| Error::new_spanned(&row, Self::UNNAMED))?;
        let marking = Self::marking(&row.attrs).ok_or_else(|| Error::new_spanned(&row.ident, Naming::UNNAMED))?;
        let naming = marking.parse_args::<Naming>()?;
        let placed = fields.iter().map(Placed::from_field).collect::<syntax::Result<Vec<Placed>>>()?;
        Ok(Record { visibility: row.vis, row: row.ident, table: naming.table, fields: placed })
    }
}
