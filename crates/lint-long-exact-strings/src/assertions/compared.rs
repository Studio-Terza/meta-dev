mod literal;

use std::cell::RefCell;
use std::collections::BTreeSet;

use syn::punctuated::Punctuated;
use syn::{Expr, Macro, Token};

use lint_layout::{Name, Whole, Word, words};

use super::bindings::Bindings;
use super::pinning::Pinning;
use literal::{Literal, Reading};

#[derive(Debug)]
pub struct Compared<'run> {
    constants: &'run Bindings,
    locals: &'run Bindings,
}

impl<'run> Compared<'run> {
    const PEELED: [Word<'static>; 8] =
        words!["to_owned", "to_string", "into", "as_str", "clone", "as_bytes", "to_vec", "as_ref"];
    const WRAPPERS: [Word<'static>; 5] = words!["Some", "Ok", "Err", "from", "new"];
    const FORMATS: [Word<'static>; 2] = words!["format", "format_args"];
    const LISTS: [Word<'static>; 2] = words!["concat", "vec"];

    pub fn new(constants: &'run Bindings, locals: &'run Bindings) -> Compared<'run> {
        Compared { constants, locals }
    }

    pub fn pinning(&self, side: &Expr) -> Option<Pinning> {
        self.traced(side, &RefCell::new(BTreeSet::new()))
    }

    fn traced(&self, side: &Expr, seen: &RefCell<BTreeSet<Name>>) -> Option<Pinning> {
        match side {
            Expr::Lit(literal) => Literal::pinning(&literal.lit, Reading::Plain),
            Expr::Reference(reference) => self.traced(&reference.expr, seen),
            Expr::Paren(inner) => self.traced(&inner.expr, seen),
            Expr::MethodCall(call) if Name::from(&call.method).matches::<Whole>(&Self::PEELED).is_some() => {
                self.traced(&call.receiver, seen)
            }
            Expr::Call(call)
                if Literal::called(&call.func).and_then(|last| last.matches::<Whole>(&Self::WRAPPERS)).is_some() =>
            {
                call.args.iter().find_map(|argument| self.traced(argument, seen))
            }
            Expr::Array(array) => array.elems.iter().find_map(|element| self.traced(element, seen)),
            Expr::Tuple(tuple) => tuple.elems.iter().find_map(|element| self.traced(element, seen)),
            Expr::Struct(built) => built.fields.iter().find_map(|field| self.traced(&field.expr, seen)),
            Expr::Macro(invocation) => self.built(&invocation.mac, seen),
            Expr::Path(path) => path.path.get_ident().map(Name::from).and_then(|name| self.bound(&name, seen)),
            _ => None,
        }
    }

    fn bound(&self, name: &Name, seen: &RefCell<BTreeSet<Name>>) -> Option<Pinning> {
        let fresh = seen.borrow_mut().insert(name.clone());
        let value = self.locals.bound(name).or_else(|| self.constants.bound(name)).filter(|_| fresh)?;
        self.traced(value, seen)
    }

    fn built(&self, invocation: &Macro, seen: &RefCell<BTreeSet<Name>>) -> Option<Pinning> {
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident))?;
        let arguments: Vec<Expr> = invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map(|arguments| arguments.into_iter().collect())
            .unwrap_or_default();
        match arguments.first() {
            Some(Expr::Lit(literal)) if name.matches::<Whole>(&Self::FORMATS).is_some() => {
                Literal::pinning(&literal.lit, Reading::Format)
            }
            _ if name.matches::<Whole>(&Self::LISTS).is_some() => {
                arguments.iter().find_map(|argument| self.traced(argument, seen))
            }
            _ => None,
        }
    }
}
