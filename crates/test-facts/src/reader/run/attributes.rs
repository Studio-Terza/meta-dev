use syn::punctuated::Punctuated;
use syn::{Attribute, Block, Expr, Ident, Meta, Stmt, Token};

use lint_layout::{Name, Whole, Word, words};

use crate::boundary::{Spelling, Truth};
use crate::claim::{Accepted, Binders, Claim, Guard, Premise, Term};

#[derive(Clone, Copy, Debug)]
enum Connective {
    Any,
    All,
    Not,
}

impl Connective {
    const NAMED: [(Word<'static>, Connective); 3] =
        [(Word::new("any"), Connective::Any), (Word::new("all"), Connective::All), (Word::new("not"), Connective::Not)];

    fn spelled(ident: &Ident) -> Option<Connective> {
        Self::NAMED.iter().find(|(word, _)| Name::from(ident) == *word).map(|(_, connective)| *connective)
    }

    fn judged(self, parts: &[Option<Truth>]) -> Option<Truth> {
        match self {
            Connective::Any => Self::settled(parts, Truth::Holds, Truth::Fails),
            Connective::All => Self::settled(parts, Truth::Fails, Truth::Holds),
            Connective::Not => parts.first().copied().flatten().map(Truth::negated),
        }
    }

    pub(crate) fn settled(parts: &[Option<Truth>], decisive: Truth, neutral: Truth) -> Option<Truth> {
        parts
            .iter()
            .find(|part| **part == Some(decisive))
            .map(|_| decisive)
            .or_else(|| parts.iter().all(|part| *part == Some(neutral)).then_some(neutral))
    }
}

struct Cfg<'meta>(&'meta Meta);

impl Cfg<'_> {
    const TEST: [Word<'static>; 1] = words!["test"];

    fn judged(&self) -> Option<Truth> {
        match self.0 {
            Meta::List(list) => list
                .path
                .segments
                .last()
                .and_then(|last| Connective::spelled(&last.ident))
                .and_then(|connective| connective.judged(&Self::parts(list))),
            Meta::Path(path) => path
                .segments
                .last()
                .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::TEST))
                .map(|_| Truth::Holds),
            Meta::NameValue(_) => None,
        }
    }

    fn parts(list: &syn::MetaList) -> Vec<Option<Truth>> {
        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .map(|parts| parts.iter().map(|part| Cfg(part).judged()).collect())
            .unwrap_or_default()
    }
}

pub(crate) struct Attributes<'attributes>(&'attributes [Attribute]);

impl<'attributes> Attributes<'attributes> {
    const CFG: [Word<'static>; 1] = words!["cfg"];
    const IGNORE: [Word<'static>; 1] = words!["ignore"];
    const SHOULD_PANIC: [Word<'static>; 1] = words!["should_panic"];
    const PANICS: Word<'static> = Word::new("Panics");

    pub(crate) fn declared(attributes: &'attributes [Attribute]) -> Attributes<'attributes> {
        Attributes(attributes)
    }

    pub(crate) fn of_statement(statement: &'attributes Stmt) -> Attributes<'attributes> {
        match statement {
            Stmt::Local(local) => Attributes(&local.attrs),
            Stmt::Macro(invocation) => Attributes(&invocation.attrs),
            Stmt::Expr(expression, _) => Attributes(Self::of_expression(expression)),
            Stmt::Item(_) => Attributes(&[]),
        }
    }

    pub(crate) fn guard(&self) -> Guard {
        self.named(&Self::IGNORE)
            .map(|_| Guard::falsified())
            .into_iter()
            .chain(self.named(&Self::CFG).map(Self::conditioned))
            .fold(Guard::default(), Guard::joined)
    }

    pub(crate) fn expecting(&self, block: &Block, binders: &Binders) -> Vec<Claim> {
        self.named(&Self::SHOULD_PANIC)
            .map(Self::expected)
            .map(|accepted| Claim::stated(binders, &self.guard(), Term::Opaque(Spelling::written(block)), accepted))
            .unwrap_or_default()
    }

    pub(crate) fn panicking(&self) -> Option<&Attribute> {
        self.named(&Self::SHOULD_PANIC)
    }

    pub(crate) fn named(&self, words: &[Word<'static>]) -> Option<&'attributes Attribute> {
        self.0.iter().find(|attribute| {
            attribute.path().segments.last().and_then(|last| Name::from(&last.ident).matches::<Whole>(words)).is_some()
        })
    }

    pub(crate) fn conditioned(attribute: &Attribute) -> Guard {
        Guard::default().joined(
            Self::predicate(attribute)
                .and_then(|meta| Cfg(&meta).judged())
                .map_or_else(|| Guard::default().conjoined(Self::unknown(attribute)), Self::settled),
        )
    }

    pub(crate) fn predicate(attribute: &Attribute) -> Option<Meta> {
        attribute.parse_args::<Meta>().ok()
    }

    pub(crate) fn settled(truth: Truth) -> Guard {
        match truth {
            Truth::Holds => Guard::default(),
            Truth::Fails => Guard::falsified(),
        }
    }

    pub(crate) fn unknown(attribute: &Attribute) -> Premise {
        Premise::new(Term::Opaque(Spelling::written(&attribute.meta)), Truth::Holds)
    }

    pub(crate) fn expected(attribute: &Attribute) -> Accepted {
        Accepted::shaped(
            Name::from(Self::PANICS),
            attribute
                .meta
                .require_list()
                .ok()
                .map(|list| Accepted::Written(Spelling::written(&list.tokens)))
                .into_iter()
                .collect(),
        )
    }

    pub(crate) fn of_expression(expression: &Expr) -> &[Attribute] {
        match expression {
            Expr::Assign(inner) => &inner.attrs,
            Expr::Binary(inner) => &inner.attrs,
            Expr::Block(inner) => &inner.attrs,
            Expr::Call(inner) => &inner.attrs,
            Expr::ForLoop(inner) => &inner.attrs,
            Expr::If(inner) => &inner.attrs,
            Expr::Loop(inner) => &inner.attrs,
            Expr::Macro(inner) => &inner.attrs,
            Expr::Match(inner) => &inner.attrs,
            Expr::MethodCall(inner) => &inner.attrs,
            Expr::Unsafe(inner) => &inner.attrs,
            Expr::While(inner) => &inner.attrs,
            _ => &[],
        }
    }
}
