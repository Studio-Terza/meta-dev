use std::iter;

use lint_layout::{Name, Whole, Word, words};

use super::body::Body;
use super::scope::{Place, Scope};
use super::trace::{Evaluation, Extent, Locals, Lowering, Quantifiers, Trace};
use crate::boundary::{Quantity, Spelling};
use crate::claim::{Binder, Cases, Claim, Domain, Draw, Ranges};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Block, Expr, Ident, Macro, Member, Pat, ReturnType, Token, Type, Visibility, parenthesized};

enum Drawn {
    Strategy(Expr),
    Declared(Type),
}

impl Drawn {
    fn ranges(&self, trace: &Trace<'_, '_>) -> Option<Ranges> {
        match self {
            Drawn::Strategy(strategy) => Extent::strategy(trace, strategy),
            Drawn::Declared(declared) => Extent::typed(declared),
        }
    }

    fn domain(&self, trace: &Trace<'_, '_>) -> Domain {
        match self {
            Drawn::Strategy(strategy) => Domain::Drawn(Lowering::term_of(trace, strategy)),
            Drawn::Declared(declared) => Domain::Drawn(crate::claim::Term::Opaque(Spelling::written(declared))),
        }
    }

    fn declared(input: ParseStream) -> syn::Result<Drawn> {
        input.parse::<Token![:]>()?;
        Ok(Drawn::Declared(input.parse()?))
    }
}

struct Input {
    pattern: Pat,
    drawn: Drawn,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Input> {
        input.call(Attribute::parse_outer)?;
        let pattern = input.call(Pat::parse_single)?;
        let strategy = input.parse::<Option<Token![in]>>()?.map(|_| input.parse::<Expr>()).transpose()?;
        let drawn = strategy.map_or_else(|| Drawn::declared(input), |strategy| Ok(Drawn::Strategy(strategy)))?;
        Ok(Input { pattern, drawn })
    }
}

struct Configured<'config>(Option<&'config Expr>);

impl Configured<'_> {
    const CASES: [Word<'static>; 2] = words!["with_cases", "cases"];

    fn draw(&self, trace: &Trace<'_, '_>) -> Draw {
        self.0.map_or(Draw::Sampled(Cases::DEFAULT), |config| {
            Configured::counted(config)
                .and_then(|count| Evaluation::quantity_of(trace, count))
                .and_then(Quantity::cases)
                .map_or_else(|| Draw::Written(Spelling::written(config)), Draw::Sampled)
        })
    }

    fn counted(config: &Expr) -> Option<&Expr> {
        match config {
            Expr::Call(call) => lint_core::Syntax::expr_path(&call.func)
                .and_then(|path| path.path.segments.last())
                .filter(|last| Name::from(&last.ident).matches::<Whole>(&Self::CASES).is_some())
                .and_then(|_| call.args.first()),
            Expr::Struct(built) => built
                .fields
                .iter()
                .find(|field| Configured::named(&field.member).matches::<Whole>(&Self::CASES).is_some())
                .map(|field| &field.expr),
            _ => None,
        }
    }

    fn named(member: &Member) -> Name {
        match member {
            Member::Named(name) => Name::from(name),
            Member::Unnamed(_) => Name::default(),
        }
    }
}

pub(crate) struct Property {
    site: Ident,
    inputs: Vec<Input>,
    body: Block,
    config: Option<Expr>,
}

impl Property {
    pub(crate) fn site(&self) -> &Ident {
        &self.site
    }

    pub(crate) fn claims(&self, scope: &Scope<'_>, place: &Place) -> Vec<Claim> {
        let locals = Locals::default();
        let empty = Quantifiers::default();
        let trace = Trace::new(scope, &locals, place, &empty);
        let draw = Configured(self.config.as_ref()).draw(&trace);
        let quantifiers = self.inputs.iter().fold(empty.clone(), |quantifiers, input| {
            let domain = input.drawn.ranges(&trace).map_or_else(|| input.drawn.domain(&trace), Domain::Ranged);
            quantifiers.quantified(Binder::new(domain, draw.clone()), &input.pattern)
        });
        Body::claims(scope, place, &self.body, &quantifiers)
    }
}

pub(crate) struct Properties {
    found: Vec<Property>,
}

impl Properties {
    const PROPTEST: Word<'static> = Word::new("proptest");
    const CONFIG: [Word<'static>; 1] = words!["proptest_config"];

    pub(crate) fn declared(invocation: &Macro) -> Vec<Property> {
        invocation
            .path
            .segments
            .last()
            .and_then(|last| Name::from(&last.ident).matches::<Whole>(&[Self::PROPTEST]))
            .and_then(|_| invocation.parse_body_with(Properties::parse).ok())
            .map_or_else(Vec::new, |properties| properties.found)
    }

    fn configured(attributes: &[Attribute]) -> Option<Expr> {
        attributes
            .iter()
            .find(|attribute| {
                attribute
                    .path()
                    .segments
                    .last()
                    .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::CONFIG))
                    .is_some()
            })
            .and_then(|attribute| attribute.parse_args::<Expr>().ok())
    }
}

impl Parse for Properties {
    fn parse(input: ParseStream) -> syn::Result<Properties> {
        let config = Properties::configured(&input.call(Attribute::parse_inner)?);
        let found = iter::from_fn(|| input.is_empty().then_some(()).map_or_else(|| Some(input.parse()), |()| None))
            .collect::<syn::Result<Vec<Property>>>()?
            .into_iter()
            .map(|property: Property| Property { config: config.clone(), ..property })
            .collect();
        Ok(Properties { found })
    }
}

impl Parse for Property {
    fn parse(input: ParseStream) -> syn::Result<Property> {
        input.call(Attribute::parse_outer)?;
        input.parse::<Visibility>()?;
        input.parse::<Token![fn]>()?;
        let site: Ident = input.parse()?;
        let arguments;
        parenthesized!(arguments in input);
        let inputs = Punctuated::<Input, Token![,]>::parse_terminated(&arguments)?.into_iter().collect();
        input.parse::<ReturnType>()?;
        Ok(Property { site, inputs, body: input.parse()?, config: None })
    }
}
