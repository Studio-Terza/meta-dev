use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::LitStr;

use crate::declared::Declared;
use crate::op::OpDerive;
use crate::resource::ResourceDerive;

struct Messages;

impl Messages {
    fn first_literal(tokens: TokenStream) -> Option<String> {
        tokens.into_iter().find_map(Self::literal_in)
    }

    fn literal_in(tree: TokenTree) -> Option<String> {
        match tree {
            TokenTree::Group(group) => Self::first_literal(group.stream()),
            TokenTree::Literal(literal) => syn::parse2::<LitStr>(quote! { #literal }).ok().map(|text| text.value()),
            _ => None,
        }
    }

    fn parsed(tokens: TokenStream) -> Option<syn::File> {
        syn::parse2(tokens).ok()
    }

    fn refusal(source: &&str) -> Option<String> {
        let item: TokenStream = source.parse().unwrap_or_default();
        Self::first_literal(Declared::expanded(TokenStream::new(), item))
    }

    fn derive_refusal(source: &&str) -> Option<String> {
        let item: TokenStream = source.parse().unwrap_or_default();
        Self::first_literal(OpDerive::expanded(item))
    }
}

#[test]
fn the_expansion_equals_the_hand_written_one() {
    let declared = quote! {
        pub trait Transcripts {
            fn sessions(&self) -> Vec<Session>;
            fn read_from(&self, session: &Session, at: usize) -> Vec<String>;
            fn lock(&self, session: &Session) -> Result<Held<'_, SessionLock>, Busy>;

            fn read_all(&self) -> Vec<Vec<String>> {
                self.sessions().iter().map(|session| self.read_from(session, 0)).collect()
            }
        }
    };
    let expected = quote! {
        pub trait Transcripts: ::port_core::Metered {
            fn sessions(&self) -> Vec<Session>;
            fn read_from(&self, session: &Session, at: usize) -> Vec<String>;
            fn lock(&self, session: &Session) -> Result<Held<'_, SessionLock>, Busy>;

            fn read_all(&self) -> Vec<Vec<String>> {
                self.sessions().iter().map(|session| self.read_from(session, 0)).collect()
            }
        }

        pub trait TranscriptsFiller {
            fn sessions(&self, pass: ::port_core::Pass<'_, transcripts_ops::Sessions>) -> Vec<Session>;
            fn read_from(&self, pass: ::port_core::Pass<'_, transcripts_ops::ReadFrom>, session: &Session, at: usize) -> Vec<String>;
            fn lock(&self, pass: ::port_core::Pass<'_, transcripts_ops::Lock>, session: &Session) -> Result<SessionLock, Busy>;
        }

        pub mod transcripts_ops {
            #[derive(Debug)]
            pub struct Sessions;

            impl ::port_core::Op for Sessions {
                const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), "::Transcripts::sessions"));
            }

            #[derive(Debug)]
            pub struct ReadFrom;

            impl ::port_core::Op for ReadFrom {
                const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), "::Transcripts::read_from"));
            }

            #[derive(Debug)]
            pub struct Lock;

            impl ::port_core::Op for Lock {
                const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), "::Transcripts::lock"));
            }
        }

        impl<'run, F: TranscriptsFiller> Transcripts for ::port_core::Port<'run, F> {
            fn sessions(&self) -> Vec<Session> {
                self.counted_call(|filler: &F, pass| filler.sessions(pass))
            }

            fn read_from(&self, session: &Session, at: usize) -> Vec<String> {
                self.counted_call(|filler: &F, pass| filler.read_from(pass, session, at))
            }

            fn lock(&self, session: &Session) -> Result<Held<'_, SessionLock>, Busy> {
                self.counted_acquire(|filler: &F, pass| filler.lock(pass, session))
            }
        }
    };
    let expansion = Messages::parsed(Declared::expanded(TokenStream::new(), declared));
    assert!(expansion.is_some());
    assert_eq!(expansion, Messages::parsed(expected));
}

#[test]
fn a_per_query_op_takes_the_query_type_as_its_pass() {
    let declared = quote! {
        trait Queries {
            #[op(Q)]
            fn ask<Q: Query>(&self, query: &Q) -> Q::Answer;
        }
    };
    let expected = quote! {
        trait Queries: ::port_core::Metered {
            fn ask<Q: Query>(&self, query: &Q) -> Q::Answer;
        }

        trait QueriesFiller {
            fn ask<Q: Query>(&self, pass: ::port_core::Pass<'_, Q>, query: &Q) -> Q::Answer;
        }

        mod queries_ops {}

        impl<'run, F: QueriesFiller> Queries for ::port_core::Port<'run, F> {
            fn ask<Q: Query>(&self, query: &Q) -> Q::Answer {
                self.counted_call(|filler: &F, pass| filler.ask(pass, query))
            }
        }
    };
    let expansion = Messages::parsed(Declared::expanded(TokenStream::new(), declared));
    assert!(expansion.is_some());
    assert_eq!(expansion, Messages::parsed(expected));
}

#[test]
fn a_resource_derive_names_the_type_and_its_release() {
    let expected = quote! {
        impl ::port_core::Resource for Lock {
            const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), "::Lock"));
            const RELEASE: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), "::Lock::release"));
        }
    };
    assert_eq!(ResourceDerive::expanded(quote! { struct Lock { held: u8 } }).to_string(), expected.to_string());
}

#[test]
fn an_op_derive_names_the_type() {
    let expected = quote! {
        impl ::port_core::Op for ByName {
            const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), "::ByName"));
        }
    };
    assert_eq!(OpDerive::expanded(quote! { struct ByName; }).to_string(), expected.to_string());
}

const HELD_ONLY: &str =
    "Held may be returned only bare, in an Option, or as the Ok of a Result, and never as a parameter";

laws::table!(
    a_malformed_port_names_what_it_refuses,
    Messages::refusal,
    [
        "trait T { fn f(&mut self); }" => Some("port methods take &self and nothing else as receiver".to_owned()),
        "trait T { fn f(self); }" => Some("port methods take &self and nothing else as receiver".to_owned()),
        "trait T { fn f(); }" => Some("port methods take &self and nothing else as receiver".to_owned()),
        "trait T { async fn f(&self); }" => Some("async, const, unsafe and extern port methods are not supported".to_owned()),
        "trait T { type Out; }" => Some("a #[port] trait holds methods only, no associated items".to_owned()),
        "trait T { const N: u8; }" => Some("a #[port] trait holds methods only, no associated items".to_owned()),
        "trait T<X> { fn f(&self); }" => Some("a #[port] trait has no generics, no supertraits and is neither unsafe nor auto".to_owned()),
        "trait T: Send { fn f(&self); }" => Some("a #[port] trait has no generics, no supertraits and is neither unsafe nor auto".to_owned()),
        "trait T { fn f<X>(&self, x: X); }" => Some("port methods take no generics; a per-query op is written #[op(Q)] with exactly one type parameter Q".to_owned()),
        "trait T { #[op(Q)] fn f<Q, R>(&self); }" => Some("port methods take no generics; a per-query op is written #[op(Q)] with exactly one type parameter Q".to_owned()),
        "trait T { fn f(&self, (a, b): (u8, u8)); }" => Some("port method parameters must be plain identifiers".to_owned()),
        "trait T { fn f(&self) -> Vec<Held<'_, R>>; }" => Some(HELD_ONLY.to_owned()),
        "trait T { fn f(&self) -> Result<R, Held<'_, R>>; }" => Some(HELD_ONLY.to_owned()),
        "trait T { fn f(&self) -> Option<Option<Held<'_, R>>>; }" => Some(HELD_ONLY.to_owned()),
        "trait T { fn f(&self, held: Held<'_, R>); }" => Some(HELD_ONLY.to_owned()),
        "trait T { #[op(Q)] fn f<Q>(&self) {} }" => Some("#[op(Q)] belongs on a required method, not a provided one".to_owned()),
    ]
);

laws::table!(
    a_malformed_derive_names_what_it_refuses,
    Messages::derive_refusal,
    ["fn not_a_type() {}" => Some("expected one of: `struct`, `enum`, `union`".to_owned())]
);
