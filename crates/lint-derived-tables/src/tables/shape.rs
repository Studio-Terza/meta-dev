mod ignored;

use std::collections::BTreeSet;

use syntax::Punctuated;
use syntax::{Arm, Block, Expr, ExprMatch, ExprReference, Lit, Macro, Pat, Path, Stmt, Token};

use invariants::Ignoring;
use lint_layout::Name;

use super::found::Table;
use ignored::Ignored;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Variant {
    owner: Name,
    variant: Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    named: Variant,
    spelling: Spelling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    Bare,
    Paired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Literal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spoken;

#[derive(Clone, Copy, Debug)]
struct Owners<'scope> {
    implementing: Option<&'scope Name>,
}

#[derive(Clone, Copy, Debug)]
struct Rows<'scope> {
    owners: Owners<'scope>,
}

#[derive(Clone, Copy, Debug)]
struct Arms<'scope> {
    owners: Owners<'scope>,
}

#[derive(Clone, Copy, Debug)]
pub struct Shape<'scope> {
    owners: Owners<'scope>,
}

struct Shared;

struct Said;

struct Listing(Vec<Entry>);

impl<'scope> Shape<'scope> {
    pub fn within(implementing: Option<&'scope Name>) -> Shape<'scope> {
        Shape { owners: Owners { implementing } }
    }

    pub fn table(self, expr: &Expr) -> Option<Table> {
        let rows = Rows { owners: self.owners };
        let entries: Option<Vec<Entry>> = Shape::array(expr)?.iter().map(|element| rows.read(element)).collect();
        entries.map(Listing).and_then(Listing::table)
    }

    pub fn mapped(self, block: &Block) -> Option<Table> {
        Some(block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Shape::matched)
            .and_then(|matched| Arms { owners: self.owners }.owner(&matched.arms))
            .map(Table::Mapped)
    }

    fn array(expr: &Expr) -> Option<&Punctuated<Expr, Token![,]>> {
        match expr {
            Expr::Array(array) => Some(&array.elems),
            Expr::Reference(reference) => Shape::referenced(reference),
            _ => None,
        }
    }

    fn referenced(reference: &ExprReference) -> Option<&Punctuated<Expr, Token![,]>> {
        match &*reference.expr {
            Expr::Array(array) => Some(&array.elems),
            _ => None,
        }
    }

    fn matched(statement: &Stmt) -> Option<&ExprMatch> {
        match statement {
            Stmt::Expr(Expr::Match(matched), None) => Some(matched),
            _ => None,
        }
    }
}

impl Rows<'_> {
    const PAIR: usize = 2;

    pub(super) fn read(self, element: &Expr) -> Option<Entry> {
        match element {
            Expr::Path(path) => self.bare(&path.path),
            Expr::Tuple(tuple) => self.paired(tuple.elems.iter().collect()),
            _ => None,
        }
    }

    fn bare(self, path: &Path) -> Option<Entry> {
        self.owners.variant(path).map(|named| Entry { named, spelling: Spelling::Bare })
    }

    fn paired(self, elements: Vec<&Expr>) -> Option<Entry> {
        let pair = Some(elements).filter(|both| both.len() == Self::PAIR)?;
        pair.iter().find_map(|element| Literal::quoted(element))?;
        let named = pair.iter().find_map(|element| Owners::path(element).and_then(|path| self.bare(path)))?;
        Some(Entry { spelling: Spelling::Paired, ..named })
    }
}

impl Listing {
    pub(super) fn table(self) -> Option<Table> {
        let found = self.0;
        found.first()?;
        let owner = Shared::common(found.iter().map(|entry| &entry.named.owner))?;
        let spelling = Shared::common(found.iter().map(|entry| &entry.spelling))?;
        let variants: BTreeSet<Name> = found.iter().map(|entry| entry.named.variant.clone()).collect();
        Some(variants).filter(|distinct| distinct.len() == found.len()).map(|distinct| spelling.table(owner, distinct))
    }
}

impl Spelling {
    fn table(self, owner: Name, variants: BTreeSet<Name>) -> Table {
        match self {
            Spelling::Bare => Table::Listed(owner, variants),
            Spelling::Paired => Table::Spelled(owner),
        }
    }
}

impl Owners<'_> {
    const SELF: &'static str = "Self";
    const OWNED: usize = 2;

    pub(super) fn variant(self, path: &Path) -> Option<Variant> {
        let named: Vec<Name> =
            path.segments.iter().rev().take(Self::OWNED).map(|segment| Name::from(&segment.ident)).collect();
        let both = Some(named).filter(|found| found.len() == Self::OWNED)?;
        let variant = both.first().cloned()?;
        let owner = both.last().cloned()?;
        let resolved =
            Some(owner).filter(|written| *written != Name::from(Self::SELF)).or_else(|| self.implementing.cloned())?;
        Some(Variant { owner: resolved, variant })
    }

    pub(super) fn path(element: &Expr) -> Option<&Path> {
        match element {
            Expr::Path(path) => Some(&path.path),
            _ => None,
        }
    }

    pub(super) fn pattern(self, pattern: &Pat) -> Option<Name> {
        match pattern {
            Pat::Or(alternatives) => self.alternatives(alternatives.cases.iter().collect()),
            _ => self.case(pattern).map(|found| found.owner),
        }
    }

    fn alternatives(self, cases: Vec<&Pat>) -> Option<Name> {
        let owners: Option<Vec<Name>> =
            cases.into_iter().map(|case| self.case(case).map(|found| found.owner)).collect();
        owners.and_then(|found| Shared::common(found.iter()))
    }

    fn case(self, pattern: &Pat) -> Option<Variant> {
        match pattern {
            Pat::Path(path) => self.variant(&path.path),
            _ => None,
        }
    }
}

impl Arms<'_> {
    pub(super) fn owner(self, arms: &[Arm]) -> Option<Name> {
        let owners: Option<Vec<Name>> = arms.iter().map(|arm| self.arm(arm)).collect();
        let found = owners.filter(|listed| listed.len() > 1)?;
        Shared::common(found.iter())
    }

    fn arm(self, arm: &Arm) -> Option<Name> {
        Some(arm)
            .filter(|candidate| candidate.guard.is_none())
            .filter(|candidate| Said::spoken(&candidate.body).is_some())
            .and_then(|candidate| self.owners.pattern(&candidate.pat))
    }
}

impl Literal {
    pub(super) fn quoted(element: &Expr) -> Option<Literal> {
        match element {
            Expr::Lit(found) => Literal::text(&found.lit),
            _ => None,
        }
    }

    fn text(lit: &Lit) -> Option<Literal> {
        match lit {
            Lit::Str(_) => Some(Literal),
            _ => None,
        }
    }
}

impl Said {
    const WRITE: &'static str = "write";
    const WRITE_STR: &'static str = "write_str";
    const WRITTEN: usize = 2;

    pub(super) fn spoken(body: &Expr) -> Option<Spoken> {
        Said::said(Said::unwrapped(body)).map(|_| Spoken)
    }

    fn unwrapped(body: &Expr) -> &Expr {
        match body {
            Expr::Block(block) => block.block.stmts.first().and_then(Said::tail).unwrap_or(body),
            _ => body,
        }
    }

    fn tail(statement: &Stmt) -> Option<&Expr> {
        match statement {
            Stmt::Expr(tail, None) => Some(tail),
            _ => None,
        }
    }

    fn said(expr: &Expr) -> Option<Literal> {
        match expr {
            Expr::Lit(_) => Literal::quoted(expr),
            Expr::MethodCall(call) => Some(call)
                .filter(|written| written.method == Self::WRITE_STR)
                .and_then(|written| written.args.first())
                .and_then(Literal::quoted),
            Expr::Macro(invoked) => Said::written(&invoked.mac),
            _ => None,
        }
    }

    fn written(invoked: &Macro) -> Option<Literal> {
        Some(invoked)
            .filter(|called| called.path.is_ident(Self::WRITE))
            .and_then(|called| {
                called.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ignoring(Ignored::Unparsed)
            })
            .filter(|parsed| parsed.len() == Self::WRITTEN)
            .and_then(|parsed| parsed.iter().nth(1).and_then(Literal::quoted))
    }
}

impl Shared {
    pub(super) fn common<'found, Found: PartialEq + Clone + 'found>(
        found: impl Iterator<Item = &'found Found>,
    ) -> Option<Found> {
        let all: Vec<&Found> = found.collect();
        let first = all.first().copied().cloned()?;
        Some(first).filter(|shared| all.iter().all(|each| *each == shared))
    }
}
