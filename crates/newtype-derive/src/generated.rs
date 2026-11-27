use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Error, Fields, Generics, Ident, Token, Variant};
use syn::{WherePredicate, parse_quote};

use crate::wrapped::{Strategy, Wrapped};

pub(crate) struct Generated<'input> {
    ident: &'input Ident,
    generics: &'input Generics,
}

struct Drawn {
    source: TokenStream,
    names: Vec<Ident>,
    required: Vec<WherePredicate>,
}

impl Generated<'_> {
    const EMPTY_ENUM: &str = "an enum with no variants has no values to generate";

    pub(crate) fn expanded(tokens: TokenStream) -> TokenStream {
        let expansion = syn::parse2(tokens).and_then(|input: DeriveInput| match &input.data {
            Data::Enum(shape) => {
                Generated { ident: &input.ident, generics: &input.generics }.enumerated(&shape.variants)
            }
            Data::Struct(_) | Data::Union(_) => Ok(Wrapped::derived::<Strategy>(input.clone())),
        });
        match expansion {
            Ok(expanded) => expanded,
            Err(error) => error.to_compile_error(),
        }
    }

    fn enumerated(&self, variants: &Punctuated<Variant, Token![,]>) -> syn::Result<TokenStream> {
        let drawn: Vec<(TokenStream, Vec<WherePredicate>)> = variants
            .iter()
            .map(|Variant { ident, fields, .. }| {
                let drawn = Drawn::from(fields);
                let members = fields.members();
                let names = &drawn.names;
                let built = quote!(Self::#ident { #(#members: #names),* });
                (drawn.generating(&built), drawn.required)
            })
            .collect();
        let strategies = drawn.iter().map(|(strategy, _)| strategy);
        let union = quote! {
            ::newtype::proptest::strategy::Union::new([
                #(::newtype::proptest::strategy::Strategy::boxed(#strategies)),*
            ])
        };
        let required = drawn.iter().flat_map(|(_, required)| required.iter().cloned()).collect();
        variants
            .first()
            .map(|_| self.implemented(required, &union))
            .ok_or_else(|| Error::new_spanned(self.ident, Self::EMPTY_ENUM))
    }

    fn implemented(&self, required: Vec<WherePredicate>, strategy: &TokenStream) -> TokenStream {
        let name = self.ident;
        let mut generics = self.generics.clone();
        let predicates = &mut generics.make_where_clause().predicates;
        predicates.push(parse_quote!(Self: ::core::fmt::Debug + 'static));
        predicates.extend(required);
        let (implemented, arguments, bounds) = generics.split_for_impl();
        quote! {
            impl #implemented ::newtype::proptest::arbitrary::Arbitrary for #name #arguments #bounds {
                type Parameters = ();
                type Strategy = ::newtype::proptest::strategy::BoxedStrategy<Self>;

                fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
                    ::newtype::proptest::strategy::Strategy::boxed(#strategy)
                }
            }
        }
    }
}

impl From<&Fields> for Drawn {
    fn from(fields: &Fields) -> Drawn {
        let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();
        Drawn {
            source: types.first().map_or_else(
                || quote!(::newtype::proptest::strategy::Just(())),
                |_| quote!((#(::newtype::proptest::arbitrary::any::<#types>(),)*)),
            ),
            names: (0..types.len()).map(|index| format_ident!("drawn{}", index)).collect(),
            required: types
                .iter()
                .flat_map(|ty| {
                    vec![
                        parse_quote!(#ty: ::newtype::proptest::arbitrary::Arbitrary + 'static),
                        parse_quote!(<#ty as ::newtype::proptest::arbitrary::Arbitrary>::Strategy: 'static),
                    ]
                })
                .collect(),
        }
    }
}

impl Drawn {
    fn generating(&self, built: &TokenStream) -> TokenStream {
        let Drawn { source, names, .. } = self;
        quote! {
            ::newtype::proptest::strategy::Strategy::prop_map(#source, |(#(#names,)*)| #built)
        }
    }
}
