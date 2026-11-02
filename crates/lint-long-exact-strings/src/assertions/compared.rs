use std::cell::RefCell;
use std::collections::BTreeSet;

use syn::punctuated::Punctuated;
use syn::{Expr, Lit, Macro, Token};

use lint_layout::{Name, Whole, Word, words};

use super::bindings::Bindings;
use super::pinning::Pinning;

pub struct Compared<'run> {
    pub constants: &'run Bindings,
    pub locals: &'run Bindings,
}

impl Compared<'_> {
    const PEELED: [Word<'static>; 8] =
        words!["to_owned", "to_string", "into", "as_str", "clone", "as_bytes", "to_vec", "as_ref"];
    const WRAPPERS: [Word<'static>; 5] = words!["Some", "Ok", "Err", "from", "new"];
    const FORMATS: [Word<'static>; 2] = words!["format", "format_args"];
    const LISTS: [Word<'static>; 2] = words!["concat", "vec"];
    const LINE_BREAK: u8 = b'\n';

    pub fn pinning(&self, side: &Expr) -> Option<Pinning> {
        self.traced(side, &RefCell::new(BTreeSet::new()))
    }

    fn traced(&self, side: &Expr, seen: &RefCell<BTreeSet<Name>>) -> Option<Pinning> {
        match side {
            Expr::Lit(literal) => Self::literal(&literal.lit),
            Expr::Reference(reference) => self.traced(&reference.expr, seen),
            Expr::Paren(inner) => self.traced(&inner.expr, seen),
            Expr::Group(inner) => self.traced(&inner.expr, seen),
            Expr::MethodCall(call) if Name::from(&call.method).matches::<Whole>(&Self::PEELED) => {
                self.traced(&call.receiver, seen)
            }
            Expr::Call(call) if Self::last(&call.func).is_some_and(|last| last.matches::<Whole>(&Self::WRAPPERS)) => {
                call.args.iter().find_map(|argument| self.traced(argument, seen))
            }
            Expr::Array(array) => array.elems.iter().find_map(|element| self.traced(element, seen)),
            Expr::Tuple(tuple) => tuple.elems.iter().find_map(|element| self.traced(element, seen)),
            Expr::Struct(built) => built.fields.iter().find_map(|field| self.traced(&field.expr, seen)),
            Expr::Macro(invocation) => self.built(&invocation.mac, seen),
            Expr::Path(path) => path.path.get_ident().map(Name::from).and_then(|name| self.bound(name, seen)),
            _ => None,
        }
    }

    fn bound(&self, name: Name, seen: &RefCell<BTreeSet<Name>>) -> Option<Pinning> {
        let fresh = seen.borrow_mut().insert(name.clone());
        let value = self.locals.bound(&name).or_else(|| self.constants.bound(&name)).filter(|_| fresh)?;
        self.traced(value, seen)
    }

    fn last(function: &Expr) -> Option<Name> {
        match function {
            Expr::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        }
    }

    fn literal(literal: &Lit) -> Option<Pinning> {
        match literal {
            Lit::Str(text) => text.value().parse().ok().filter(|pinning| *pinning == Pinning::LineBreak),
            Lit::ByteStr(bytes) => bytes.value().contains(&Self::LINE_BREAK).then_some(Pinning::LineBreak),
            _ => None,
        }
    }

    fn built(&self, invocation: &Macro, seen: &RefCell<BTreeSet<Name>>) -> Option<Pinning> {
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident))?;
        let arguments: Vec<Expr> = invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map(|arguments| arguments.into_iter().collect())
            .unwrap_or_default();
        match arguments.first() {
            Some(Expr::Lit(literal)) if name.matches::<Whole>(&Self::FORMATS) => Self::formatted(&literal.lit),
            _ if name.matches::<Whole>(&Self::LISTS) => {
                arguments.iter().find_map(|argument| self.traced(argument, seen))
            }
            _ => None,
        }
    }

    fn formatted(literal: &Lit) -> Option<Pinning> {
        match literal {
            Lit::Str(text) => text.value().parse().ok().filter(|pinning| *pinning != Pinning::Plain),
            _ => None,
        }
    }
}
