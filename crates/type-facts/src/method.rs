use syn::{ImplItem, Signature, TraitItem};

use lint_core::Walk;
use lint_layout::Name;

use crate::method_shape::MethodShape;
use crate::spelled::Spelled;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Method {
    name: Name,
    returns: Option<Name>,
    shape: MethodShape,
}

impl Method {
    pub(crate) fn of_signature(signature: &Signature, selfish: Option<&Name>) -> Self {
        let returns = Spelled::returned_by(&signature.output).and_then(Spelled::last_of);
        let shape = MethodShape::of_signature(signature, selfish);
        Self { name: Name::from(&signature.ident), returns, shape }
    }

    pub(crate) fn of_trait_item(item: &TraitItem) -> Option<Self> {
        match item {
            TraitItem::Fn(function) => {
                (Walk::outside_tests(&function.attrs) == Walk::Entered).then(|| Self::of_signature(&function.sig, None))
            }
            _ => None,
        }
    }

    pub(crate) fn of_impl_item(item: &ImplItem, selfish: Option<&Name>) -> Option<Self> {
        match item {
            ImplItem::Fn(function) => Some(Self::of_signature(&function.sig, selfish)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn returns(&self) -> Option<&Name> {
        self.returns.as_ref()
    }

    #[must_use]
    pub const fn shape(&self) -> &MethodShape {
        &self.shape
    }
}

impl AsRef<Name> for Method {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
