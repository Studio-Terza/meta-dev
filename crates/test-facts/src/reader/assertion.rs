mod table;

use syn::parse::{Parse, ParseStream};
use syn::{Attribute, BinOp, Expr, Ident, Macro};

use law_facts::Kind;
use lint_core::{MacroAliases, MacroCalls, Macros};
use lint_layout::{Name, Whole, Word, words};

use super::scope::Scope;
use super::trace::Trace;
use crate::check::{Check, Domain, Relation};
use table::Table;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Assertion {
    Equality,
    Inequality,
    Condition,
    Golden,
    Table,
    Other,
}

impl Assertion {
    const EQUALITIES: [Word<'static>; 2] = words!["assert_eq", "prop_assert_eq"];
    const INEQUALITIES: [Word<'static>; 2] = words!["assert_ne", "prop_assert_ne"];
    const CONDITIONS: [Word<'static>; 2] = words!["assert", "prop_assert"];
    const GOLDENS: [Word<'static>; 1] = words!["golden"];
    const PATTERNS: [Word<'static>; 1] = words!["matches"];

    pub(crate) fn named(invocation: &Macro, tables: &MacroAliases) -> Assertion {
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default();
        name.matches::<Whole>(&Self::EQUALITIES)
            .map(|_| Assertion::Equality)
            .or_else(|| name.matches::<Whole>(&Self::INEQUALITIES).map(|_| Assertion::Inequality))
            .or_else(|| name.matches::<Whole>(&Self::CONDITIONS).map(|_| Assertion::Condition))
            .or_else(|| name.matches::<Whole>(&Self::GOLDENS).map(|_| Assertion::Golden))
            .or_else(|| tables.call(&name).map(|_| Assertion::Table))
            .unwrap_or(Assertion::Other)
    }

    pub(crate) fn titled(invocation: &Macro) -> Option<Name> {
        invocation.parse_body_with(Titled::parse).ok().map(|titled| Name::from(&titled.name))
    }

    pub(crate) fn checks(self, invocation: &Macro, trace: &Trace<'_, '_>, domain: Domain) -> Vec<Check> {
        match self {
            Assertion::Equality | Assertion::Golden => trace.operands(Relation::Equal, invocation, domain),
            Assertion::Inequality => trace.operands(Relation::Differs, invocation, domain),
            Assertion::Condition => Self::condition(invocation, trace, domain),
            Assertion::Table | Assertion::Other => Vec::new(),
        }
    }

    pub(crate) fn nested(self, invocation: &Macro) -> Vec<Macro> {
        match self {
            Assertion::Other => MacroCalls::within(invocation),
            Assertion::Equality
            | Assertion::Inequality
            | Assertion::Condition
            | Assertion::Golden
            | Assertion::Table => Vec::new(),
        }
    }

    pub(crate) fn law_checks(self, invocation: &Macro, scope: &Scope<'_>, tables: &MacroAliases) -> Vec<Check> {
        match self {
            Assertion::Table => Table::checks(invocation, scope),
            Assertion::Other => Table::law(invocation, Kind::of_macro(invocation), scope, tables),
            Assertion::Equality | Assertion::Inequality | Assertion::Condition | Assertion::Golden => Vec::new(),
        }
    }

    fn condition(invocation: &Macro, trace: &Trace<'_, '_>, domain: Domain) -> Vec<Check> {
        Macros::arguments(invocation)
            .into_iter()
            .take(1)
            .flat_map(|condition| Self::tested(&condition, trace, domain))
            .collect()
    }

    fn tested(condition: &Expr, trace: &Trace<'_, '_>, domain: Domain) -> Vec<Check> {
        match condition {
            Expr::Binary(binary) => {
                trace.compared(Self::comparison(binary.op), &[&*binary.left, &*binary.right], domain)
            }
            Expr::Macro(inner) => trace.compared(Self::shaped(&inner.mac), &[condition], domain),
            _ => trace.compared(Relation::Holds, &[condition], domain),
        }
    }

    fn shaped(inner: &Macro) -> Relation {
        let name = inner.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default();
        name.matches::<Whole>(&Self::PATTERNS).map_or(Relation::Holds, |_| Relation::Pattern)
    }

    fn comparison(operator: BinOp) -> Relation {
        match operator {
            BinOp::Eq(_) => Relation::Equal,
            BinOp::Ne(_) => Relation::Differs,
            _ => Relation::Holds,
        }
    }
}

struct Titled {
    name: Ident,
}

impl Parse for Titled {
    fn parse(input: ParseStream) -> syn::Result<Titled> {
        input.call(Attribute::parse_outer)?;
        let name = input.parse()?;
        input.parse::<proc_macro2::TokenStream>()?;
        Ok(Titled { name })
    }
}
