use invariants::Ignoring;
use proc_macro2::{TokenStream, TokenTree};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, ItemImpl, Meta, MetaList, Token};

use crate::ignored::Ignored;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Gate {
    Entered,
    Skipped,
}

impl Gate {
    pub(crate) fn of(attributes: &[Attribute]) -> Gate {
        attributes.iter().map(Gate::of_attribute).find(|gate| *gate == Gate::Skipped).unwrap_or(Gate::Entered)
    }

    pub(crate) fn inherent(item: &ItemImpl) -> Gate {
        match item.trait_ {
            None => Gate::Entered,
            Some(_) => Gate::Skipped,
        }
    }

    pub(crate) fn entered(self) -> Option<()> {
        (self == Gate::Entered).then_some(())
    }

    pub(crate) fn and(self, inner: Gate) -> Gate {
        self.entered().map_or(Gate::Skipped, |()| inner)
    }

    fn of_attribute(attribute: &Attribute) -> Gate {
        Some(attribute)
            .filter(|candidate| candidate.path().is_ident("cfg"))
            .and_then(|cfg| cfg.parse_args::<Meta>().ignoring(Ignored::Unparsed))
            .map_or(Gate::Entered, |condition| Gate::of_condition(&condition))
    }

    fn of_condition(condition: &Meta) -> Gate {
        match condition {
            Meta::Path(path) => {
                Some(path).filter(|named| named.is_ident("test")).map_or(Gate::Entered, |_| Gate::Skipped)
            }
            Meta::List(list) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ignoring(Ignored::Unparsed)
                .map_or(Gate::Entered, |nested| Gate::joined(list, &nested)),
            Meta::NameValue(_) => Gate::Entered,
        }
    }

    fn joined(list: &MetaList, nested: &Punctuated<Meta, Token![,]>) -> Gate {
        let gates: Vec<Gate> = nested.iter().map(Gate::of_condition).collect();
        Gate::deciding(list)
            .and_then(|deciding| gates.iter().find(|gate| **gate == deciding).or(gates.first()))
            .copied()
            .unwrap_or(Gate::Entered)
    }

    fn deciding(list: &MetaList) -> Option<Gate> {
        let all = list.path.is_ident("all").then_some(Gate::Skipped);
        all.or_else(|| list.path.is_ident("any").then_some(Gate::Entered))
    }
}

#[derive(Debug)]
pub(crate) struct TestMark;

impl TestMark {
    const NAMES: [&'static str; 3] = ["test", "rstest", "test_case"];

    pub(crate) fn on(attribute: &Attribute) -> Option<&Attribute> {
        let last = attribute.path().segments.last();
        last.filter(|segment| TestMark::NAMES.iter().any(|name| segment.ident == name)).map(|_| attribute)
    }

    pub(crate) fn among(attributes: &[Attribute]) -> Option<&Attribute> {
        attributes.iter().find_map(TestMark::on)
    }

    pub(crate) fn written(tokens: TokenStream) -> Vec<Attribute> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let here = trees.iter().zip(trees.iter().skip(1)).filter_map(|(mark, body)| TestMark::attribute(mark, body));
        let nested = trees.iter().filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(TestMark::written(group.stream())),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        });
        here.chain(nested.flatten()).collect()
    }

    fn attribute(mark: &TokenTree, body: &TokenTree) -> Option<Attribute> {
        let written = TokenStream::from_iter([mark.clone(), body.clone()]);
        Attribute::parse_outer.parse2(written).ignoring(Ignored::Unparsed).and_then(|parsed| parsed.into_iter().next())
    }
}
