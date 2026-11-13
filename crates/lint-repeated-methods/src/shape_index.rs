mod inners;
mod method_shape;
mod placed;
mod repeated_shape;

use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{ItemImpl, ItemMod, Visibility};

use lint_core::{Bound, DeclaredType, Home, Parsed, Site, Syntax, TypeHomes, Violation, Walk};
use lint_file::Concluded;
use lint_layout::{Length, LineNumber, Name};

use super::RepeatedMethods;
use inners::Inners;
use method_shape::{Erasure, MethodShape, SpelledMethod};
use placed::Placed;
use repeated_shape::RepeatedShape;

#[derive(Debug)]
pub struct SpelledImpl<'head> {
    home: Home<'head>,
    written: Name,
    methods: Vec<SpelledMethod<'head>>,
}

#[derive(Debug)]
pub struct ImplSpellings<'head> {
    homes: TypeHomes<'head>,
    inners: Inners<'head>,
    impls: Vec<SpelledImpl<'head>>,
}

impl<'head> From<Parsed<'_, 'head>> for ImplSpellings<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> ImplSpellings<'head> {
        let mut reading = ImplReading { parsed, impls: Vec::new() };
        reading.visit_file(parsed.file().syntax());
        ImplSpellings { homes: TypeHomes::from(parsed), inners: Inners::from(parsed), impls: reading.impls }
    }
}

#[derive(Debug)]
pub struct ShippedImpls<'head>(Option<ImplSpellings<'head>>);

impl<'head> From<Parsed<'_, 'head>> for ShippedImpls<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> ShippedImpls<'head> {
        ShippedImpls(parsed.file().shipped().map(|_| ImplSpellings::from(parsed)))
    }
}

#[derive(Default)]
struct Gathered<'head> {
    homes: Vec<TypeHomes<'head>>,
    inners: Vec<Inners<'head>>,
    impls: Vec<SpelledImpl<'head>>,
}

impl<'head> Gathered<'head> {
    fn including(mut self, part: ImplSpellings<'head>) -> Gathered<'head> {
        self.homes.push(part.homes);
        self.inners.push(part.inners);
        self.impls.extend(part.impls);
        self
    }
}

impl<'head> FromIterator<ShippedImpls<'head>> for Concluded<'head> {
    fn from_iter<Parts: IntoIterator<Item = ShippedImpls<'head>>>(parts: Parts) -> Concluded<'head> {
        let gathered = parts.into_iter().filter_map(|part| part.0).fold(Gathered::default(), Gathered::including);
        let homes: TypeHomes<'head> = gathered.homes.into_iter().collect();
        let inners: Inners<'head> = gathered.inners.into_iter().collect();
        let owners = gathered.impls.iter().fold(BTreeMap::new(), |mut owners, spelled| {
            let owner = homes.resolve(spelled.written.clone(), spelled.home);
            let inner = inners.inner_of(&owner.home(), owner.as_ref());
            let shapes = spelled.methods.iter().fold(BTreeMap::new(), |mut shapes, method| {
                shapes.entry(method.shaped(inner)).or_insert(*method.as_ref());
                shapes
            });
            Placed::new(owner, shapes).recorded_in(&mut owners);
            owners
        });
        Owners(owners).repeated(RepeatedMethods::TYPES_PER_SHAPE).into_iter().collect()
    }
}

struct Owners<'head>(BTreeMap<DeclaredType<'head>, BTreeMap<MethodShape, Site<'head>>>);

impl<'head> Owners<'head> {
    fn repeated(&self, most: Bound) -> Vec<Violation<'head>> {
        let shapes = self
            .0
            .iter()
            .flat_map(|(owner, methods)| methods.iter().map(move |(shape, site)| (shape, owner, *site)))
            .fold(
                BTreeMap::new(),
                |mut shapes: BTreeMap<&MethodShape, BTreeMap<&DeclaredType<'head>, Site<'head>>>,
                 (shape, owner, site)| {
                    shapes.entry(shape).or_default().insert(owner, site);
                    shapes
                },
            );
        shapes
            .into_iter()
            .filter(|(_, types)| most.allows(Length::new(types.len())).is_none())
            .filter_map(|(shape, types)| {
                let site = types.values().min().copied()?;
                Some(Violation::new(site, RepeatedShape::new(shape, types.into_keys().collect())))
            })
            .collect()
    }
}

struct ImplReading<'file, 'head> {
    parsed: Parsed<'file, 'head>,
    impls: Vec<SpelledImpl<'head>>,
}

impl<'ast> Visit<'ast> for ImplReading<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let head = self.parsed.head();
        let written = Name::of_type(&item.self_ty).filter(|_| Walk::outside_tests(&item.attrs) == Walk::Entered);
        let spelled = written.map(|name| {
            let erasure = Erasure::around(&name);
            let methods = Syntax::inherent(item)
                .into_iter()
                .flat_map(|inherent| inherent.items.iter().filter_map(Syntax::impl_fn))
                .filter(|function| matches!(function.vis, Visibility::Public(_) | Visibility::Restricted(_)))
                .filter(|function| function.sig.constness.is_none())
                .map(|function| {
                    let site = Site::new(head.path(), LineNumber::from(function.sig.ident.span()));
                    SpelledMethod::written(&function.sig, &erasure, site)
                })
                .collect();
            SpelledImpl { home: Home::from(head), written: name.clone(), methods }
        });
        self.impls.extend(spelled);
    }
}
