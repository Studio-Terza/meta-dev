use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Field, Fields, Ident, ItemStruct, LitStr, Token, Type, Visibility};

use crate::{Origin, Table};

#[derive(Debug)]
pub struct Recorded {
    visibility: Visibility,
    row: Ident,
    table: LitStr,
    fields: Vec<Placed>,
}

#[derive(Debug)]
pub struct Placed {
    name: Ident,
    column: Type,
    origin: Origin,
}

#[derive(Debug)]
pub struct Invoked {
    row: Option<Type>,
    source: LitStr,
    rest: TokenStream,
}

#[derive(Debug)]
struct Naming {
    table: LitStr,
}

#[derive(Clone, Copy, Debug)]
struct Marked {
    origin: Origin,
}

impl Recorded {
    pub const KEYLESS: &'static str = "a record has one field marked #[record(key)] or #[record(generated_key)]";
    const UNNAMED: &'static str = "a record is a struct with named fields";
    const ATTRIBUTE: &'static str = "record";

    #[must_use]
    pub fn marked(row: &ItemStruct) -> Option<syn::Result<Recorded>> {
        Recorded::marking(&row.attrs).map(|marking| Recorded::placed(row, marking))
    }

    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    #[must_use]
    pub fn row(&self) -> &Ident {
        &self.row
    }

    #[must_use]
    pub fn table(&self) -> &LitStr {
        &self.table
    }

    #[must_use]
    pub fn fields(&self) -> &[Placed] {
        &self.fields
    }

    #[must_use]
    pub fn statements(&self) -> [LitStr; 5] {
        let described = self.fields.iter().fold(Table::named(&self.table.value()), |described, field| {
            described.column(&field.name.to_string(), field.origin)
        });
        described.statements().map(|sql| LitStr::new(&sql, self.table.span()))
    }

    fn marking(attributes: &[Attribute]) -> Option<&Attribute> {
        attributes.iter().find(|attribute| attribute.path().is_ident(Recorded::ATTRIBUTE))
    }

    fn placed(row: &ItemStruct, marking: &Attribute) -> syn::Result<Recorded> {
        let naming = marking.parse_args::<Naming>()?;
        let named = match &row.fields {
            Fields::Named(named) => Ok(named),
            Fields::Unnamed(_) | Fields::Unit => Err(Error::new_spanned(row, Recorded::UNNAMED)),
        }?;
        let fields = named.named.iter().map(Placed::from_field).collect::<syn::Result<Vec<Placed>>>()?;
        Ok(Recorded { visibility: row.vis.clone(), row: row.ident.clone(), table: naming.table, fields })
    }
}

impl Placed {
    #[must_use]
    pub fn name(&self) -> &Ident {
        &self.name
    }

    #[must_use]
    pub fn column(&self) -> &Type {
        &self.column
    }

    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    #[must_use]
    pub fn overwrite(&self, table: &LitStr) -> LitStr {
        LitStr::new(&Table::named(&table.value()).overwrite(&self.name.to_string()), table.span())
    }

    fn from_field(field: &Field) -> syn::Result<Placed> {
        let name = field.ident.clone().ok_or_else(|| Error::new_spanned(field, Recorded::UNNAMED))?;
        let marked = Recorded::marking(&field.attrs).map(Attribute::parse_args::<Marked>).transpose()?;
        Ok(Placed { name, column: field.ty.clone(), origin: marked.map_or(Origin::Given, |found| found.origin) })
    }
}

impl Invoked {
    #[must_use]
    pub fn row(&self) -> Option<&Type> {
        self.row.as_ref()
    }

    #[must_use]
    pub fn source(&self) -> &LitStr {
        &self.source
    }

    #[must_use]
    pub fn rest(&self) -> &TokenStream {
        &self.rest
    }

    fn typed(input: ParseStream<'_>) -> syn::Result<Option<Type>> {
        let typed = Some(()).filter(|()| !input.peek(LitStr));
        typed
            .map(|()| {
                let row = input.parse::<Type>()?;
                input.parse::<Token![,]>()?;
                Ok(row)
            })
            .transpose()
    }
}

impl Naming {
    const TABLE: &'static str = "table";
    const UNNAMED: &'static str = "a record names its table, as in #[record(table = \"cursors\")]";
}

impl Parse for Recorded {
    fn parse(input: ParseStream<'_>) -> syn::Result<Recorded> {
        let row: ItemStruct = input.parse()?;
        Recorded::marked(&row).unwrap_or_else(|| Err(Error::new_spanned(&row.ident, Naming::UNNAMED)))
    }
}

impl Parse for Invoked {
    fn parse(input: ParseStream<'_>) -> syn::Result<Invoked> {
        let row = Invoked::typed(input)?;
        let source = input.parse()?;
        input.parse::<Option<Token![,]>>()?;
        Ok(Invoked { row, source, rest: input.parse()? })
    }
}

impl Parse for Naming {
    fn parse(input: ParseStream<'_>) -> syn::Result<Naming> {
        let word: Ident = input.parse()?;
        Some(&word).filter(|found| *found == Naming::TABLE).ok_or_else(|| Error::new(word.span(), Naming::UNNAMED))?;
        input.parse::<Token![=]>()?;
        Ok(Naming { table: input.parse()? })
    }
}

impl Parse for Marked {
    fn parse(input: ParseStream<'_>) -> syn::Result<Marked> {
        let word: Ident = input.parse()?;
        let origin = word.to_string().parse::<Origin>().map_err(|failed| Error::new(word.span(), failed))?;
        Ok(Marked { origin })
    }
}
