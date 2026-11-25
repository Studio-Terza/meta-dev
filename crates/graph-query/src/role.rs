use std::str::FromStr;

use graph::{MethodShape, Receiver, Returned, Taken};
use graph_reader::ReceiverSyntax;
use lint_core::Syntax;
use lint_layout::Name;
use syntax::{FnArg, Past, ReturnType, Signature, Type, TypeParamBound, TypeSpine};

use crate::boundary::QueryFailure;
use crate::score::Score;

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(from = Role)]
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
    fn taken(ty: &Type) -> Taken {
        let peeled = TypeSpine::from(ty).past(|node| matches!(node, Type::Reference(_) | Type::Paren(_)));
        peeled
            .and_then(|found| Syntax::type_path(found.node()))
            .and_then(|path| path.path.segments.last())
            .map(|segment| Name::from(&segment.ident))
            .and_then(|name| Returned::named(&name))
            .filter(|returned| *returned == Returned::Itself)
            .map_or(Taken::Other, |_| Taken::Itself)
    }

    fn returned(ty: &Type) -> Returned {
        let peeled = TypeSpine::from(ty).past(|node| matches!(node, Type::Reference(_) | Type::Paren(_)));
        peeled.map_or(Returned::Value, Self::shaped)
    }

    fn shaped(peeled: Past<'_, Type>) -> Returned {
        match peeled.node() {
            Type::Tuple(tuple) => tuple.elems.first().map_or(Returned::Nothing, |_| Returned::Value),
            Type::Slice(_) | Type::Array(_) => Returned::Many,
            Type::ImplTrait(bounds) => bounds.bounds.iter().find_map(Self::bounded).unwrap_or(Returned::Value),
            Type::Path(path) => path
                .path
                .segments
                .last()
                .and_then(|segment| Returned::named(&Name::from(&segment.ident)))
                .unwrap_or(Returned::Value),
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
        syntax::parse::parse_str::<Signature>(text)
            .map(|signature| Role::from(&signature))
            .map_err(|failure| QueryFailure::Role { text: text.to_owned(), reason: failure.to_string() })
    }
}
