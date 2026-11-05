mod condition;
mod operands;
mod table;

use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, Macro};

use law_facts::Kind;
use lint_core::{MacroAliases, MacroCalls, Macros};
use lint_layout::{Name, Whole, Word, words};

use super::scope::{Place, Scope};
use super::trace::Trace;
use crate::claim::Claim;
use condition::Condition;
use operands::{Comparing, Operands};
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

    pub(crate) fn claims(self, invocation: &Macro, trace: &Trace<'_, '_>) -> Vec<Claim> {
        match self {
            Assertion::Equality | Assertion::Golden => {
                Operands::read(invocation).map(|operands| operands.claims(trace, Comparing::Equal)).unwrap_or_default()
            }
            Assertion::Inequality => Operands::read(invocation)
                .map(|operands| operands.claims(trace, Comparing::Unequal))
                .unwrap_or_default(),
            Assertion::Condition => Assertion::condition(invocation, trace),
            Assertion::Table | Assertion::Other => Vec::new(),
        }
    }

    fn condition(invocation: &Macro, trace: &Trace<'_, '_>) -> Vec<Claim> {
        Macros::arguments(invocation).first().map(|condition| Condition::claims(trace, condition)).unwrap_or_default()
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

    pub(crate) fn law_claims(self, invocation: &Macro, scope: &Scope<'_>, place: &Place) -> Vec<Claim> {
        match self {
            Assertion::Table => Table::claims(invocation, scope, place),
            Assertion::Other => Table::law(invocation, Kind::of_macro(invocation), scope, place),
            Assertion::Equality | Assertion::Inequality | Assertion::Condition | Assertion::Golden => Vec::new(),
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
