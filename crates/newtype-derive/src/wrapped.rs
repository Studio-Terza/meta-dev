mod check;
mod only_field;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_quote};

use check::Check;
use only_field::OnlyField;

pub struct Wrapped {
    input: DeriveInput,
    field: OnlyField,
    check: Option<Check>,
}

impl Wrapped {
    pub fn expanded(tokens: TokenStream, expansion: fn(Wrapped) -> TokenStream) -> TokenStream {
        Self::parsed(tokens).map_or_else(|error| error.to_compile_error(), expansion)
    }

    pub fn constructors(&self) -> TokenStream {
        let name = &self.input.ident;
        let (implemented, arguments, bounds) = self.input.generics.split_for_impl();
        let member = self.field.member();
        let inner = self.field.inner();
        match &self.check {
            None => quote! {
                impl #implemented ::newtype::Newtype for #name #arguments #bounds {
                    type Inner = #inner;

                    fn new(inner: Self::Inner) -> Self {
                        Self { #member: inner }
                    }

                    fn into_inner(self) -> Self::Inner {
                        self.#member
                    }
                }
            },
            Some(check) => {
                let function = check.function();
                let error = check.error();
                quote! {
                    impl #implemented ::newtype::Validated for #name #arguments #bounds {
                        type Inner = #inner;
                        type Error = #error;

                        fn try_new(inner: Self::Inner) -> ::core::result::Result<Self, Self::Error> {
                            #function(&inner)?;
                            ::core::result::Result::Ok(Self { #member: inner })
                        }

                        fn into_inner(self) -> Self::Inner {
                            self.#member
                        }
                    }
                }
            }
        }
    }

    pub fn arbitrary(mut self) -> TokenStream {
        let name = &self.input.ident;
        let inner = self.field.inner();
        let predicates = &mut self.input.generics.make_where_clause().predicates;
        predicates.push(parse_quote!(#inner: ::proptest::arbitrary::Arbitrary + 'static));
        predicates.push(parse_quote!(<#inner as ::proptest::arbitrary::Arbitrary>::Strategy: 'static));
        let (implemented, arguments, bounds) = self.input.generics.split_for_impl();
        let strategy = self.check.as_ref().map_or_else(Self::plain_strategy, |_| Check::strategy());
        quote! {
            impl #implemented ::proptest::arbitrary::Arbitrary for #name #arguments #bounds {
                type Parameters = <#inner as ::proptest::arbitrary::Arbitrary>::Parameters;
                type Strategy = ::proptest::strategy::BoxedStrategy<Self>;

                fn arbitrary_with(parameters: Self::Parameters) -> Self::Strategy {
                    let inner = ::proptest::arbitrary::any_with::<#inner>(parameters);
                    ::proptest::strategy::Strategy::boxed(#strategy)
                }
            }
        }
    }

    fn plain_strategy() -> TokenStream {
        quote! {
            ::proptest::strategy::Strategy::prop_map(inner, <Self as ::newtype::Newtype>::new)
        }
    }

    fn parsed(tokens: TokenStream) -> syn::Result<Wrapped> {
        let input: DeriveInput = syn::parse2(tokens)?;
        let check = Check::declared(&input.attrs)?;
        let field = OnlyField::found_in(&input)?;
        Ok(Wrapped { input, field, check })
    }
}
