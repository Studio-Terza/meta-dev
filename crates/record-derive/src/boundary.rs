mod reading;

use syntax::TokenStream;
use syntax::{Data, DataStruct, DeriveInput, Error, Fields, Ident};
use syntax::{format_ident, quote};

use reading::Reading;

#[derive(Debug)]
pub struct Record {
    ident: Ident,
    readings: Vec<Reading>,
}

impl TryFrom<DeriveInput> for Record {
    type Error = Error;

    fn try_from(input: DeriveInput) -> syntax::Result<Record> {
        Some(()).filter(|()| input.generics.params.is_empty()).ok_or_else(|| {
            Error::new_spanned(&input.generics, "a record has no generic parameters; read a concrete line")
        })?;
        let named = Record::named(&input.data)
            .ok_or_else(|| Error::new_spanned(&input.ident, "a record is a struct with named fields"))?;
        let readings = named.iter().map(Reading::declared).collect::<syntax::Result<Vec<Reading>>>()?;
        Ok(Record { ident: input.ident, readings })
    }
}

impl From<&Record> for TokenStream {
    fn from(record: &Record) -> TokenStream {
        record.generated()
    }
}

impl Record {
    fn named(data: &Data) -> Option<&syntax::Punctuated<syntax::Field, syntax::Token![,]>> {
        match data {
            Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => Some(&named.named),
            Data::Struct(_) | Data::Enum(_) | Data::Union(_) => None,
        }
    }

    fn generated(&self) -> TokenStream {
        let name = &self.ident;
        let stages: Vec<Ident> =
            (0..=self.readings.len()).map(|stage| format_ident!("{}Read{}", name, stage)).collect();
        let declarations = stages.iter().enumerate().map(|(stage, ident)| self.stage(ident, stage));
        let pairs = stages.iter().zip(stages.iter().skip(1)).zip(&self.readings);
        let steps = pairs.enumerate().map(|(stage, ((from, into), reading))| self.step(from, into, stage, reading));
        let start = stages.first();
        let fields: Vec<&Ident> = self.readings.iter().map(Reading::ident).collect();
        let advances = self.readings.iter().map(|_| quote!(.step()?));
        quote! {
            const _: () = {
                #(#declarations)*
                #(#steps)*
                impl ::grammar_output::Record for #name {
                    fn read(
                        fields: ::grammar_output::Fields,
                    ) -> ::core::result::Result<
                        ::grammar_output::Parsed<Self, ::grammar_output::Fields>,
                        ::grammar_output::Unread,
                    > {
                        let __last = #start { __fields: fields } #(#advances)*;
                        ::core::result::Result::Ok(::grammar_output::Parsed::new(
                            #name { #(#fields: __last.#fields),* },
                            __last.__fields,
                        ))
                    }
                }
            };
        }
    }

    fn stage(&self, ident: &Ident, stage: usize) -> TokenStream {
        let held = self.readings.iter().take(stage);
        let names = held.clone().map(Reading::ident);
        let types = held.map(Reading::field_type);
        quote! {
            struct #ident {
                #(#names: #types,)*
                __fields: ::grammar_output::Fields,
            }
        }
    }

    fn step(&self, from: &Ident, into: &Ident, stage: usize, reading: &Reading) -> TokenStream {
        let stepped = reading.stepped(into, self.readings.iter().take(stage).map(Reading::ident));
        quote! {
            impl #from {
                fn step(self) -> ::core::result::Result<#into, ::grammar_output::Unread> {
                    #stepped
                }
            }
        }
    }
}
