use syntax::parse::parse_quote;
use syntax::{Data, DeriveInput, Error, Expr, Fields, Generics, Ident, Punctuated, Token, TokenStream, Variant};
use syntax::{WherePredicate, format_ident, quote};

use crate::settings::{Checking, Expressed, Settings};
use crate::wrapped::{Strategy, Wrapped};

pub struct Generated<'input> {
    ident: &'input Ident,
    generics: &'input Generics,
}

enum Generator<'settings> {
    Shaped,
    Constructed(&'settings Expr, Checking),
    Escaped(&'settings Expr),
}

struct Drawn {
    source: TokenStream,
    names: Vec<Ident>,
    required: Vec<WherePredicate>,
}

impl Generated<'_> {
    const CONSTRUCTED_ENUM: &str = "`from` names a struct's constructor; an enum generates its variants";
    const EMPTY_ENUM: &str = "an enum with no variants has no values to generate";

    pub fn expanded(tokens: TokenStream) -> TokenStream {
        syntax::parse::parse2(tokens)
            .and_then(|input: DeriveInput| {
                Settings::declared(&input.attrs).and_then(|settings| Generator::from(&settings).routed(&input))
            })
            .unwrap_or_else(Error::into_compile_error)
    }

    fn escaped(&self, generate: &Expr) -> TokenStream {
        self.implemented(Vec::new(), &quote!(#generate()))
    }

    fn constructed(&self, from: &Expr, checking: Checking, fields: &Fields) -> TokenStream {
        let drawn = Drawn::from(fields);
        let names = &drawn.names;
        let strategy = drawn.generating(&quote!(#from(#(#names),*)), checking);
        self.implemented(drawn.required, &strategy)
    }

    fn enumerated(&self, variants: &Punctuated<Variant, Token![,]>) -> syntax::Result<TokenStream> {
        let drawn: Vec<(TokenStream, Vec<WherePredicate>)> = variants
            .iter()
            .map(|Variant { ident, fields, .. }| {
                let drawn = Drawn::from(fields);
                let members = fields.members();
                let names = &drawn.names;
                let built = quote!(Self::#ident { #(#members: #names),* });
                (drawn.generating(&built, Checking::Unchecked), drawn.required)
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

impl<'input> From<&'input DeriveInput> for Generated<'input> {
    fn from(input: &'input DeriveInput) -> Generated<'input> {
        Generated { ident: &input.ident, generics: &input.generics }
    }
}

impl<'settings> From<&'settings Settings> for Generator<'settings> {
    fn from(settings: &'settings Settings) -> Generator<'settings> {
        settings.expressed(Expressed::Generate).map_or_else(
            || {
                settings
                    .expressed(Expressed::From)
                    .map_or(Generator::Shaped, |from| Generator::Constructed(from, settings.checking()))
            },
            Generator::Escaped,
        )
    }
}

impl Generator<'_> {
    fn routed(self, input: &DeriveInput) -> syntax::Result<TokenStream> {
        match self {
            Generator::Escaped(generate) => Ok(Generated::from(input).escaped(generate)),
            Generator::Constructed(from, checking) => Generator::constructing(from, checking, input),
            Generator::Shaped => Generator::shaping(input),
        }
    }

    fn constructing(from: &Expr, checking: Checking, input: &DeriveInput) -> syntax::Result<TokenStream> {
        match &input.data {
            Data::Struct(shape) => Ok(Generated::from(input).constructed(from, checking, &shape.fields)),
            Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(&input.ident, Generated::CONSTRUCTED_ENUM)),
        }
    }

    fn shaping(input: &DeriveInput) -> syntax::Result<TokenStream> {
        match &input.data {
            Data::Enum(shape) => Generated::from(input).enumerated(&shape.variants),
            Data::Struct(shape) => Ok(Generator::structured(input, &shape.fields)),
            Data::Union(_) => Ok(Wrapped::derived::<Strategy>(input.clone())),
        }
    }

    fn structured(input: &DeriveInput, fields: &Fields) -> TokenStream {
        let lone = fields.iter().next().filter(|only| only.ident.is_none()).filter(|_| fields.len() == 1);
        lone.map_or_else(|| Drawn::from(fields).filled(input, fields), |_| Wrapped::derived::<Strategy>(input.clone()))
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
    fn filled(self, input: &DeriveInput, fields: &Fields) -> TokenStream {
        let members = fields.members();
        let names = &self.names;
        let strategy = self.generating(&quote!(Self { #(#members: #names),* }), Checking::Unchecked);
        Generated::from(input).implemented(self.required, &strategy)
    }

    fn generating(&self, built: &TokenStream, checking: Checking) -> TokenStream {
        let Drawn { source, names, .. } = self;
        let kept = Drawn::kept(built, checking);
        quote! {
            ::newtype::proptest::strategy::Strategy::prop_filter_map(
                #source,
                "the constructor refuses this value",
                |(#(#names,)*)| #kept,
            )
        }
    }

    fn kept(built: &TokenStream, checking: Checking) -> TokenStream {
        match checking {
            Checking::Unchecked => quote!(::core::option::Option::Some(#built)),
            Checking::Checked => quote! {
                match #built {
                    ::core::result::Result::Ok(made) => ::core::option::Option::Some(made),
                    ::core::result::Result::Err(_) => ::core::option::Option::None,
                }
            },
        }
    }
}
