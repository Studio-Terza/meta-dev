use syn::{ImplItem, Signature, TraitItem};

use lint_core::Walk;
use lint_layout::Name;

use crate::arity::Arity;
use crate::method_shape::MethodShape;
use crate::spelled::Spelled;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Method {
    name: Name,
    arity: Arity,
    returns: Option<Name>,
    shape: MethodShape,
}

impl Method {
    pub(crate) fn of_signature(signature: &Signature, selfish: Option<&Name>) -> Method {
        let returns = Spelled::returned_by(&signature.output).and_then(Spelled::last_of);
        let shape = MethodShape::of_signature(signature, selfish);
        Method { name: Name::from(&signature.ident), arity: Arity::new(signature.inputs.len()), returns, shape }
    }

    pub(crate) fn of_trait_item(item: &TraitItem) -> Option<Method> {
        match item {
            TraitItem::Fn(function) => (Walk::outside_tests(&function.attrs) == Walk::Entered)
                .then(|| Method::of_signature(&function.sig, None)),
            _ => None,
        }
    }

    pub(crate) fn of_impl_item(item: &ImplItem, selfish: Option<&Name>) -> Option<Method> {
        match item {
            ImplItem::Fn(function) => Some(Method::of_signature(&function.sig, selfish)),
            _ => None,
        }
    }

    #[must_use]
    pub fn arity(&self) -> Arity {
        self.arity
    }

    #[must_use]
    pub fn returns(&self) -> Option<&Name> {
        self.returns.as_ref()
    }

    #[must_use]
    pub fn shape(&self) -> &MethodShape {
        &self.shape
    }
}

impl AsRef<Name> for Method {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
