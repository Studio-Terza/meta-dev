use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed, Index, Member, Type};

pub struct OnlyField {
    member: Member,
    inner: Type,
}

impl OnlyField {
    const REFUSED: &str = "a newtype is a struct with exactly one field";
    const ONLY: usize = 1;
    const FIRST: u32 = 0;

    pub fn taken(data: Data) -> Result<OnlyField, Data> {
        match data {
            Data::Struct(DataStruct { struct_token, fields, semi_token }) => Self::among(fields)
                .map_err(|returned| Data::Struct(DataStruct { struct_token, fields: returned, semi_token })),
            Data::Enum(_) | Data::Union(_) => Err(data),
        }
    }

    pub fn refused(input: DeriveInput) -> Error {
        Error::new_spanned(input, Self::REFUSED)
    }

    pub fn member(&self) -> &Member {
        &self.member
    }

    pub fn inner(&self) -> &Type {
        &self.inner
    }

    fn among(fields: Fields) -> Result<OnlyField, Fields> {
        match fields {
            Fields::Named(FieldsNamed { brace_token, named }) => {
                Self::sole(named).map_err(|returned| Fields::Named(FieldsNamed { brace_token, named: returned }))
            }
            Fields::Unnamed(FieldsUnnamed { paren_token, unnamed }) => Self::sole(unnamed)
                .map_err(|returned| Fields::Unnamed(FieldsUnnamed { paren_token, unnamed: returned })),
            Fields::Unit => Err(fields),
        }
    }

    fn sole<Separator: Default>(
        fields: Punctuated<Field, Separator>,
    ) -> Result<OnlyField, Punctuated<Field, Separator>> {
        <[Field; Self::ONLY]>::try_from(fields.into_iter().collect::<Vec<Field>>())
            .map(|[field]| OnlyField::from(field))
            .map_err(|returned| returned.into_iter().collect())
    }
}

impl From<Field> for OnlyField {
    fn from(field: Field) -> OnlyField {
        OnlyField {
            member: field
                .ident
                .map_or_else(|| Member::Unnamed(Index { index: Self::FIRST, span: field.ty.span() }), Member::Named),
            inner: field.ty,
        }
    }
}
