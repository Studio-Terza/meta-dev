mod reading;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Data, DataStruct, DeriveInput, Error, Fields, Ident, Type};

use reading::Reading;

#[derive(Debug)]
pub struct Record {
    ident: Ident,
    source: Type,
    readings: Vec<Reading>,
}

impl TryFrom<DeriveInput> for Record {
    type Error = Error;

    fn try_from(input: DeriveInput) -> syn::Result<Record> {
        Some(()).filter(|()| input.generics.params.is_empty()).ok_or_else(|| {
            Error::new_spanned(&input.generics, "a record has no generic parameters; read a concrete line")
        })?;
        let named = Record::named(&input.data)
            .ok_or_else(|| Error::new_spanned(&input.ident, "a record is a struct with named fields"))?;
        let readings = named.iter().map(Reading::declared).collect::<syn::Result<Vec<Reading>>>()?;
        let declared = input.attrs.iter().find(|attribute| attribute.path().is_ident("record"));
        let source = declared.map_or_else(
            || Ok(syn::parse_quote!(::grammar_output::Fields)),
            |attribute| Sourced::try_from(attribute).map(|sourced| sourced.source),
        )?;
        Ok(Record { ident: input.ident, source, readings })
    }
}

impl From<&Record> for TokenStream {
    fn from(record: &Record) -> TokenStream {
        record.generated()
    }
}

#[derive(Debug)]
struct Sourced {
    source: Type,
}

impl TryFrom<&syn::Attribute> for Sourced {
    type Error = Error;

    fn try_from(attribute: &syn::Attribute) -> syn::Result<Sourced> {
        attribute.parse_args_with(|input: syn::parse::ParseStream| {
            let key: Ident = input.parse()?;
            Some(())
                .filter(|()| key == "source")
                .ok_or_else(|| Error::new_spanned(&key, "the only option is source = Type"))?;
            input.parse::<syn::Token![=]>()?;
            Ok(Sourced { source: input.parse()? })
        })
    }
}

impl Record {
    fn named(data: &Data) -> Option<&Punctuated<syn::Field, syn::Token![,]>> {
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
        let source = &self.source;
        quote! {
            const _: () = {
                #(#declarations)*
                #(#steps)*
                impl ::grammar_output::Record<#source> for #name {
                    fn read(
                        fields: #source,
                    ) -> ::core::result::Result<
                        ::grammar_output::Parsed<Self, #source>,
                        <#source as ::grammar_output::Source>::Unread,
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
        let source = &self.source;
        quote! {
            struct #ident {
                #(#names: #types,)*
                __fields: #source,
            }
        }
    }

    fn step(&self, from: &Ident, into: &Ident, stage: usize, reading: &Reading) -> TokenStream {
        let stepped = reading.stepped(into, self.readings.iter().take(stage).map(Reading::ident));
        let source = &self.source;
        quote! {
            impl #from {
                fn step(self) -> ::core::result::Result<#into, <#source as ::grammar_output::Source>::Unread> {
                    #stepped
                }
            }
        }
    }
}
