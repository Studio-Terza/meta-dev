mod ground;

use std::collections::BTreeSet;
use std::mem;

use proc_macro2::TokenTree;
use syn::{Attribute, Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro, Signature, Variant, Visibility};

use lint_core::{SourceFile, Walk};
use std::path::Path;

use lint_layout::Name;
use lint_placement::PackageRoot;

use crate::FileMentions;
use lint_sweep::{Address, Descend, Gathers, Kind, Sweep, Watch, Watching};

pub(crate) use ground::Ground;

#[derive(Debug, Default)]
pub(crate) struct Named {
    shipped: BTreeSet<Name>,
    tested: BTreeSet<Name>,
}

impl Named {
    fn names(&mut self, ground: Ground) -> &mut BTreeSet<Name> {
        match ground {
            Ground::Shipped => &mut self.shipped,
            Ground::Tests => &mut self.tested,
        }
    }

    pub(crate) fn mentioned<'head>(self, path: &'head Path, package: &'head PackageRoot) -> FileMentions<'head> {
        FileMentions { path, package, shipped: self.shipped, tested: self.tested }
    }

    fn absorbed(mut self, module: Named) -> Named {
        self.shipped.extend(module.shipped);
        self.tested.extend(module.tested);
        self
    }
}

#[derive(Debug)]
struct Grounds {
    ground: Ground,
    outer: Vec<Ground>,
}

impl Grounds {
    fn entered(&mut self, attributes: &[Attribute]) -> Descend {
        let kept = (Walk::outside_tests(attributes) == Walk::Entered).then_some(self.ground);
        self.outer.push(mem::replace(&mut self.ground, kept.unwrap_or(Ground::Tests)));
        Descend::Into
    }

    fn left(&mut self) {
        self.ground = self.outer.pop().unwrap_or(self.ground);
    }
}

#[derive(Debug, Default)]
struct Skips(Vec<Address>);

impl Skips {
    fn passed(&mut self, at: Address) -> Descend {
        let place = self.0.iter().rposition(|skipped| *skipped == at);
        place.map_or(Descend::Into, |index| {
            self.0.swap_remove(index);
            Descend::Over
        })
    }
}

#[derive(Debug)]
pub struct Mentions {
    grounds: Grounds,
    skips: Skips,
    named: Named,
}

impl Mentions {
    pub(crate) fn of_file(file: &SourceFile) -> Mentions {
        let ground = file.shipped().map_or(Ground::Tests, |_| Ground::Shipped);
        let tested =
            file.test_modules().iter().map(|module| Sweep::gathered(module, Mentions::grounded(Ground::Tests)));
        let named = tested.fold(Named::default(), Named::absorbed);
        Mentions { named, ..Mentions::grounded(ground) }
    }

    fn grounded(ground: Ground) -> Mentions {
        Mentions { grounds: Grounds { ground, outer: Vec::new() }, skips: Skips::default(), named: Named::default() }
    }

    fn function(&mut self, attributes: &[Attribute], visibility: &Visibility) -> Descend {
        self.skips.0.extend(attributes.iter().map(Address::from));
        self.skips.0.push(Address::from(visibility));
        self.grounds.entered(attributes)
    }

    fn scanned(&mut self, tree: TokenTree) {
        match tree {
            TokenTree::Ident(ident) => self.named.names(self.grounds.ground).extend([Name::from(&ident)]),
            TokenTree::Group(group) => group.stream().into_iter().for_each(|token| self.scanned(token)),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

impl<'ast> Watch<'ast> for Mentions {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Ident,
            Kind::Macro,
            Kind::ItemMod,
            Kind::ItemFn,
            Kind::ItemImpl,
            Kind::ImplItemFn,
            Kind::Signature,
            Kind::Variant,
            Kind::Attribute,
            Kind::Visibility,
        ]
    }

    fn on_ident(&mut self, ident: &'ast Ident) -> Descend {
        let named = self.skips.passed(Address::from(ident)).entered().map(|_| Name::from(ident));
        self.named.names(self.grounds.ground).extend(named);
        Descend::Into
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        invocation.tokens.clone().into_iter().for_each(|tree| self.scanned(tree));
        Descend::Into
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        self.grounds.entered(&module.attrs)
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.grounds.left();
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.function(&function.attrs, &function.vis)
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn) {
        self.grounds.left();
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        self.grounds.entered(&item.attrs)
    }

    fn off_item_impl(&mut self, _: &'ast ItemImpl) {
        self.grounds.left();
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.function(&function.attrs, &function.vis)
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn) {
        self.grounds.left();
    }

    fn on_signature(&mut self, signature: &'ast Signature) -> Descend {
        self.skips.0.push(Address::from(&signature.ident));
        Descend::Into
    }

    fn on_variant(&mut self, variant: &'ast Variant) -> Descend {
        self.skips.0.push(Address::from(&variant.ident));
        self.skips.0.extend(variant.attrs.iter().map(Address::from));
        Descend::Into
    }

    fn on_attribute(&mut self, attribute: &'ast Attribute) -> Descend {
        self.skips.passed(Address::from(attribute))
    }

    fn on_visibility(&mut self, visibility: &'ast Visibility) -> Descend {
        self.skips.passed(Address::from(visibility))
    }
}

impl Gathers<Named> for Mentions {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Named {
        self.named
    }
}
