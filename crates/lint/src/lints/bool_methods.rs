use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemTrait, ReturnType, Signature, TraitItemFn, Type};

use crate::finding::{Findings, Found};
use crate::gate::Gate;
use crate::hooks::{Descent, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;

#[derive(Debug, Default)]
pub(crate) struct BoolMethods {
    found: Vec<Found>,
}

impl BoolMethods {
    const FIX: &'static str =
        "return an enum to match on, an Option or Result carrying the value, or a proof type such as NonEmpty";

    fn opened(&mut self, attributes: &[Attribute], signature: &Signature) -> Option<()> {
        Gate::of(attributes).entered()?;
        let flag = match &signature.output {
            ReturnType::Type(_, returned) => match &**returned {
                Type::Path(path) => (path.qself.is_none() && path.path.is_ident("bool")).then_some(()),
                _ => None,
            },
            ReturnType::Default => None,
        };
        let name = &signature.ident;
        self.found
            .extend(flag.map(|()| Found::spanned(name.span(), format_args!("fn {name} -> bool"), BoolMethods::FIX)));
        Some(())
    }
}

impl Hooks for BoolMethods {
    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::inherent(item).and(Gate::of(&item.attrs)).entered())
    }

    fn item_trait(&mut self, item: &ItemTrait) -> Descent {
        Descent::from(Gate::of(&item.attrs).entered())
    }

    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        Descent::from(self.opened(&function.attrs, &function.sig))
    }

    fn impl_item_fn(&mut self, function: &ImplItemFn) -> Descent {
        Descent::from(self.opened(&function.attrs, &function.sig))
    }

    fn trait_item_fn(&mut self, function: &TraitItemFn) -> Descent {
        Descent::from(self.opened(&function.attrs, &function.sig))
    }
}

impl<'file> Lint<'file> for BoolMethods {
    const HEADLINE: &'static str =
        "a function returning bool forgets what it found and makes the caller branch on a flag";
    const READS: Reads = Reads::Shipped;

    type Fact = Findings;

    fn new(_: &'file Source) -> BoolMethods {
        BoolMethods::default()
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(BoolMethods::HEADLINE, source.shown(), self.found)
    }
}
