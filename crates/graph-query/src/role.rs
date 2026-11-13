use std::str::FromStr;

use graph::{MethodShape, Receiver, Returned, Taken};
use graph_reader::ReceiverSyntax;
use lint_layout::Name;
use syn::{FnArg, ReturnType, Signature, Type, TypeParamBound};

use crate::boundary::QueryFailure;
use crate::score::Score;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Role(MethodShape);

#[derive(Clone, Copy, Debug)]
struct Spelled;

#[derive(Clone, Copy, Debug)]
struct Written;

impl Role {
    #[must_use]
    pub fn agreement(&self, other: &MethodShape) -> Score {
        let wanted = &self.0;
        let parts = [
            wanted.receiver() == other.receiver(),
            wanted.takes().len() == other.takes().len(),
            wanted.takes() == other.takes(),
            wanted.returned() == other.returned(),
        ];
        Score::new(parts.into_iter().filter(|agrees| *agrees).count())
    }
}

impl Spelled {
    fn last(ty: &Type) -> Option<Name> {
        match ty {
            Type::Reference(reference) => Self::last(&reference.elem),
            Type::Paren(paren) => Self::last(&paren.elem),
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }

    fn taken(ty: &Type) -> Taken {
        Self::last(ty)
            .and_then(|name| Returned::named(&name))
            .filter(|returned| *returned == Returned::Itself)
            .map_or(Taken::Other, |_| Taken::Itself)
    }

    fn returned(ty: &Type) -> Returned {
        match ty {
            Type::Reference(reference) => Self::returned(&reference.elem),
            Type::Paren(paren) => Self::returned(&paren.elem),
            Type::Tuple(tuple) => tuple.elems.first().map_or(Returned::Nothing, |_| Returned::Value),
            Type::Slice(_) | Type::Array(_) => Returned::Many,
            Type::ImplTrait(bounds) => bounds.bounds.iter().find_map(Self::bounded).unwrap_or(Returned::Value),
            Type::Path(_) => Self::last(ty).and_then(|name| Returned::named(&name)).unwrap_or(Returned::Value),
            _ => Returned::Value,
        }
    }

    fn bounded(bound: &TypeParamBound) -> Option<Returned> {
        match bound {
            TypeParamBound::Trait(found) => {
                found.path.segments.last().and_then(|segment| Returned::named(&Name::from(&segment.ident)))
            }
            _ => None,
        }
    }
}

impl Written {
    fn taken(input: &FnArg) -> Option<Taken> {
        match input {
            FnArg::Typed(typed) => Some(Spelled::taken(&typed.ty)),
            FnArg::Receiver(_) => None,
        }
    }

    fn output(output: &ReturnType) -> Returned {
        match output {
            ReturnType::Type(_, ty) => Spelled::returned(ty),
            ReturnType::Default => Returned::Nothing,
        }
    }
}

impl From<&Signature> for Role {
    fn from(signature: &Signature) -> Role {
        Role(MethodShape::new(
            signature.receiver().map(|written| Receiver::from(ReceiverSyntax::from(written))),
            signature.inputs.iter().filter_map(Written::taken).collect(),
            Written::output(&signature.output),
        ))
    }
}

impl AsRef<MethodShape> for Role {
    fn as_ref(&self) -> &MethodShape {
        &self.0
    }
}

impl FromStr for Role {
    type Err = QueryFailure;

    fn from_str(text: &str) -> Result<Role, QueryFailure> {
        syn::parse_str::<Signature>(text)
            .map(|signature| Role::from(&signature))
            .map_err(|failure| QueryFailure::Role { text: text.to_owned(), reason: failure.to_string() })
    }
}
