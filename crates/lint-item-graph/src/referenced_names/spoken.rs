use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Fields, Item, Variant};

#[derive(Debug, Default)]
pub struct Spoken(TokenStream);

impl Spoken {
    pub fn of_item(item: &Item) -> TokenStream {
        match item {
            Item::Struct(found) => {
                Self::default()
                    .appended(&found.attrs)
                    .appended([&found.generics])
                    .appended(&found.generics.where_clause)
                    .fields(&found.fields)
                    .0
            }
            Item::Enum(found) => {
                found
                    .variants
                    .iter()
                    .fold(
                        Self::default()
                            .appended(&found.attrs)
                            .appended([&found.generics])
                            .appended(&found.generics.where_clause),
                        Self::variant,
                    )
                    .0
            }
            _ => item.to_token_stream(),
        }
    }

    fn appended(mut self, parts: impl IntoIterator<Item = impl ToTokens>) -> Self {
        parts.into_iter().for_each(|part| part.to_tokens(&mut self.0));
        self
    }

    fn fields(self, fields: &Fields) -> Self {
        fields.iter().fold(self, |spoken, field| spoken.appended(&field.attrs).appended([&field.ty]))
    }

    fn variant(self, variant: &Variant) -> Self {
        let discriminant = variant.discriminant.as_ref().map(|(_, value)| value);
        self.appended(&variant.attrs).fields(&variant.fields).appended(discriminant)
    }
}
