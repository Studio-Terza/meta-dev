mod reading;

use std::collections::BTreeSet;

use proc_macro2::{TokenStream, TokenTree};
use syn::Macro;

use graph::{Building, Law, LawKind, Operation, OperationOwner, Property, Route, Side, TypeRef};
use lint_layout::{Name, TestMarks, Whole, Word, words};

use super::Lowering;
use super::seat::{Built, Footing, Located};
use super::signature::Written;
use reading::Reading;

#[derive(Clone, Debug)]
enum LoweredOwner {
    Typed(TypeRef<Written>),
    Operand,
}

#[derive(Debug)]
struct LoweredOperation {
    owner: LoweredOwner,
    member: Name,
}

#[derive(Debug)]
struct LoweredRoute {
    side: Side,
    operation: Option<LoweredOperation>,
    fallback: Option<LoweredOwner>,
}

#[derive(Debug)]
pub(super) struct LawRow {
    located: Located,
    kind: LawKind,
    title: Name,
    mentions: BTreeSet<Name>,
    routes: Vec<LoweredRoute>,
}

#[derive(Clone, Copy, Debug)]
struct Kinds;

#[derive(Clone, Copy, Debug)]
struct Mentions;

impl Kinds {
    const ROUNDTRIP: [Word<'static>; 1] = words!["roundtrip"];
    const PROPTEST: [Word<'static>; 1] = words!["proptest"];
    const PROPERTIES: [Word<'static>; 8] = words![
        "associative",
        "commutative",
        "identity",
        "inverse",
        "idempotent",
        "absorbing",
        "distributive",
        "preserves"
    ];
    const NAMED: [Property; 8] = [
        Property::Associative,
        Property::Commutative,
        Property::Identity,
        Property::Inverse,
        Property::Idempotent,
        Property::Absorbing,
        Property::Distributive,
        Property::Preserves,
    ];

    fn of_macro(invocation: &Macro) -> Option<LawKind> {
        let last = invocation.path.segments.last().map(|segment| Name::from(&segment.ident));
        let law = TestMarks::law(invocation);
        let roundtrip =
            last.as_ref().and_then(|name| name.matches::<Whole>(&Self::ROUNDTRIP)).and(law).map(|_| LawKind::Roundtrip);
        let proptest = last.as_ref().and_then(|name| name.matches::<Whole>(&Self::PROPTEST)).map(|_| LawKind::Proptest);
        let property = law.and(last.as_ref()).and_then(Self::property).map(LawKind::Property);
        roundtrip.or(proptest).or(property).or(law.map(|_| LawKind::Law))
    }

    fn property(name: &Name) -> Option<Property> {
        Self::PROPERTIES
            .iter()
            .zip(Self::NAMED)
            .find(|(word, _)| *name == Name::from(**word))
            .map(|(_, property)| property)
    }
}

impl Mentions {
    const SEPARATOR: char = ',';

    fn beyond_title(tokens: TokenStream, kind: LawKind) -> BTreeSet<Name> {
        match kind.titled() {
            Some(_) => Self::named(tokens.into_iter().skip_while(|tree| Self::separator(tree).is_none()).skip(1)),
            None => Self::named(tokens.into_iter()),
        }
    }

    fn named(trees: impl Iterator<Item = TokenTree>) -> BTreeSet<Name> {
        trees.flat_map(Self::tree_names).collect()
    }

    fn tree_names(tree: TokenTree) -> BTreeSet<Name> {
        match tree {
            TokenTree::Ident(ident) => BTreeSet::from([Name::from(&ident)]),
            TokenTree::Group(group) => Self::named(group.stream().into_iter()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => BTreeSet::new(),
        }
    }

    fn separator(tree: &TokenTree) -> Option<&TokenTree> {
        match tree {
            TokenTree::Punct(punct) => (punct.as_char() == Self::SEPARATOR).then_some(tree),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }
}

impl LoweredOwner {
    fn resolved(self, footing: &Footing<'_>, located: &Located) -> OperationOwner {
        match self {
            LoweredOwner::Typed(ty) => {
                OperationOwner::Typed(Written::typed(&ty, &footing.resolving(), located.scope()))
            }
            LoweredOwner::Operand => OperationOwner::Operand,
        }
    }
}

impl LoweredRoute {
    fn resolved(self, footing: &Footing<'_>, located: &Located) -> Route {
        let operation =
            self.operation.map(|found| Operation::new(found.owner.resolved(footing, located), found.member));
        Route::new(self.side, operation, self.fallback.map(|owner| owner.resolved(footing, located)))
    }
}

impl Built for LawRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let located = self.located;
        let routes = self.routes.into_iter().map(|route| route.resolved(&footing, &located)).collect();
        located.law(footing, building, Law::new(self.kind, self.title, self.mentions, routes))
    }
}

impl Lowering<'_> {
    pub(super) fn law_row(&self, invocation: &Macro) -> Option<LawRow> {
        Kinds::of_macro(invocation).map(|kind| {
            let title = invocation.path.segments.last().map_or_default(|last| Name::from(&last.ident));
            LawRow {
                located: self.here.located(lint_layout::Position::from(syn::spanned::Spanned::span(&invocation.path))),
                kind,
                title,
                mentions: Mentions::beyond_title(invocation.tokens.clone(), kind),
                routes: Reading::new(&self.generics).routes(invocation, kind),
            }
        })
    }
}
