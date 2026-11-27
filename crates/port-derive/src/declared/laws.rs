use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{GenericArgument, PathArguments, ReturnType, Type, TypePath, TypeReference};

use super::boundary::Spelling;
use super::method::{Method, Parameter};
use super::{Declared, Rendered};

pub(crate) struct Laws;

struct Kept {
    reply: TokenStream,
    settled: TokenStream,
}

struct Replayed {
    variant: TokenStream,
    reply: TokenStream,
    arm: TokenStream,
}

impl Rendered for Laws {
    fn rendered(declared: &Declared) -> TokenStream {
        let visibility = declared.visibility();
        let caller = declared.caller_name();
        let named = &declared.sealed_trait().ident;
        let call = format_ident!("{named}Call");
        let reply = format_ident!("{named}Reply");
        let replayed: Vec<Replayed> = declared
            .as_ref()
            .iter()
            .filter(|method| method.op_struct().is_some() && method.returned().replayable().is_some())
            .map(|method| Laws::replayed(method, &call, &reply, caller))
            .collect();
        let variants = replayed.iter().map(|each| &each.variant);
        let replies = replayed.iter().map(|each| &each.reply);
        let arms = replayed.iter().map(|each| &each.arm);
        quote! {
            #[cfg(feature = "laws")]
            #[derive(Debug, ::serde::Deserialize)]
            #visibility enum #call {
                #(#variants),*
            }

            #[cfg(feature = "laws")]
            #[derive(Debug, PartialEq)]
            #visibility enum #reply {
                #(#replies),*
            }

            #[cfg(feature = "laws")]
            impl #call {
                pub fn replayed(&self, port: &impl #caller) -> #reply {
                    match self {
                        #(#arms),*
                    }
                }
            }
        }
    }
}

impl Laws {
    pub(crate) fn named(declared: &Declared) -> TokenStream {
        let named = &declared.sealed_trait().ident;
        let call = format_ident!("{named}Call");
        let reply = format_ident!("{named}Reply");
        quote! {
            #[cfg(feature = "laws")]
            pub use super::{#call as Call, #reply as Reply};
        }
    }

    fn replayed(method: &Method, call: &syn::Ident, reply: &syn::Ident, caller: &impl quote::ToTokens) -> Replayed {
        let name = method.name();
        let variant = Spelling::camel(name);
        let parameters = method.parameters();
        let owned = parameters.iter().map(|parameter| Laws::owned(parameter.tree()));
        let bound: Vec<_> = parameters.iter().map(Parameter::name).collect();
        let arguments = parameters.iter().map(Laws::lent);
        let Kept { reply: kept, settled } = Laws::kept(method.output());
        let fields = parameters.first().map(|_| quote!((#(#owned),*)));
        let pattern = parameters.first().map(|_| quote!((#(#bound),*)));
        Replayed {
            variant: quote!(#variant #fields),
            reply: quote!(#variant(#kept)),
            arm: quote! {
                #call::#variant #pattern => {
                    let answered = #caller::#name(port #(, #arguments)*);
                    #reply::#variant(#settled)
                }
            },
        }
    }

    fn owned(tree: &Type) -> TokenStream {
        Laws::lent_type(tree).map_or_else(|| quote!(#tree), |elem| quote!(<#elem as ::std::borrow::ToOwned>::Owned))
    }

    fn lent(parameter: &Parameter) -> TokenStream {
        let name = parameter.name();
        Laws::lent_type(parameter.tree()).map_or_else(
            || quote!(::core::clone::Clone::clone(#name)),
            |elem| quote!(::std::borrow::Borrow::<#elem>::borrow(#name)),
        )
    }

    fn lent_type(tree: &Type) -> Option<&Type> {
        if let Type::Reference(TypeReference { elem, .. }) = tree { Some(elem) } else { None }
    }

    fn kept(output: &ReturnType) -> Kept {
        match output {
            ReturnType::Default => Kept { reply: quote!(()), settled: quote!(answered) },
            ReturnType::Type(_, tree) => Laws::kept_type(tree),
        }
    }

    fn kept_type(tree: &Type) -> Kept {
        Laws::succeeded(tree).map_or_else(
            || Kept { reply: quote!(#tree), settled: quote!(answered) },
            |value| Kept {
                reply: quote!(::core::option::Option<#value>),
                settled: quote! {
                    match answered {
                        ::core::result::Result::Ok(value) => ::core::option::Option::Some(value),
                        ::core::result::Result::Err(_failed) => ::core::option::Option::None,
                    }
                },
            },
        )
    }

    fn succeeded(tree: &Type) -> Option<&Type> {
        let Type::Path(TypePath { qself: None, path }) = tree else { return None };
        let segment = path.segments.last().filter(|segment| segment.ident == "Result")?;
        let PathArguments::AngleBracketed(angled) = &segment.arguments else { return None };
        angled.args.iter().find_map(|argument| match argument {
            GenericArgument::Type(value) => Some(value),
            _ => None,
        })
    }
}
