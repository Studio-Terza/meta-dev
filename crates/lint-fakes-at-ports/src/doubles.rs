mod finding;

use std::collections::BTreeMap;
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemUnion, Path as TraitPath};

use lint_file::Concluded;

use lint_core::{FileHead, Parsed, Site, Violation, Walk};
use lint_layout::{CamelWord, LineNumber, Name, UsePath, Word, words};
use lint_paths::{Imports, Module, Nested};

use crate::kind::Kind;
use crate::port_traits::PortTraits;
use finding::Finding;

#[derive(Debug, Default)]
pub struct Doubles<'file> {
    declared: BTreeMap<DoubleKey<'file>, Site<'file>>,
    implemented: Vec<Implemented<'file>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DoubleKey<'file> {
    target: &'file Path,
    name: Name,
}

#[derive(Debug)]
struct Implemented<'file> {
    double: DoubleKey<'file>,
    site: Site<'file>,
    written: UsePath,
    kind: Kind,
}

impl<'file> Doubles<'file> {
    pub fn scanned(parsed: Parsed<'_, 'file>, imports: &Imports<'_>, ports: &PortTraits) -> Doubles<'file> {
        let head = parsed.head();
        let mut doubles = Doubles::default();
        let mut reader = Reader {
            head,
            target: head.placement().crate_root().directory(),
            module: Module::of_head(head),
            imports,
            ports,
            doubles: &mut doubles,
        };
        reader.visit_file(parsed.file().syntax());
        doubles
    }

    fn violations(self) -> Vec<Violation<'file>> {
        let unported = self
            .declared
            .iter()
            .filter(|(double, _)| {
                self.implemented
                    .iter()
                    .filter(|implemented| implemented.double == **double)
                    .all(|implemented| implemented.kind != Kind::Port)
            })
            .map(|(double, site)| Violation::new(*site, Finding::Unported(&double.name)));
        let beside = self.implemented.iter().filter(|implemented| implemented.kind == Kind::Other).map(|implemented| {
            Violation::new(implemented.site, Finding::Beside(&implemented.double.name, implemented.written.clone()))
        });
        let mut violations: Vec<Violation<'file>> = unported.chain(beside).collect();
        violations.sort_by(|left, right| left.site().cmp(right.site()));
        violations
    }
}

struct Reader<'file, 'run> {
    head: &'file FileHead,
    target: &'file Path,
    module: Module<'file>,
    imports: &'run Imports<'run>,
    ports: &'run PortTraits,
    doubles: &'run mut Doubles<'file>,
}

impl<'file> Reader<'file, '_> {
    const DOUBLES: [Word<'static>; 5] = words!["fake", "stub", "mock", "spy", "dummy"];

    fn declared(&mut self, name: &Ident) {
        let double = DoubleKey { target: self.target, name: Name::from(name) };
        let site = Site::new(self.head.path(), LineNumber::from(name.span()));
        self.doubles.declared.extend(double.name.matches::<CamelWord>(&Self::DOUBLES).map(|_| (double, site)));
    }

    fn double_of(&self, item: &ItemImpl) -> Option<DoubleKey<'file>> {
        Name::of_type(&item.self_ty)
            .filter(|name| name.matches::<CamelWord>(&Self::DOUBLES).is_some())
            .map(|name| DoubleKey { target: self.target, name })
    }

    fn implementation(&self, double: DoubleKey<'file>, path: &TraitPath) -> Implemented<'file> {
        let written = UsePath::from(path);
        let bare = Kind::written(&written);
        let candidates = self.imports.candidates(&self.module, written.clone());
        let line = path.segments.first().map_or(LineNumber::FIRST, |first| LineNumber::from(first.ident.span()));
        Implemented {
            double,
            site: Site::new(self.head.path(), line),
            kind: bare.judged(&candidates, self.ports),
            written,
        }
    }
}

impl<'file> Nested<'file> for Reader<'file, '_> {
    fn module(&mut self) -> &mut Module<'file> {
        &mut self.module
    }
}

impl<'ast> Visit<'ast> for Reader<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.nested(module, |reader| visit::visit_item_mod(reader, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || {
            let found = self
                .double_of(item)
                .zip(item.trait_.as_ref())
                .map(|(double, (_, path, _))| self.implementation(double, path));
            self.doubles.implemented.extend(found);
        });
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || self.declared(&item.ident));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || self.declared(&item.ident));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        Walk::enter(&item.attrs, || self.declared(&item.ident));
    }
}

impl<'file> FromIterator<Doubles<'file>> for Concluded<'file> {
    fn from_iter<Parts: IntoIterator<Item = Doubles<'file>>>(parts: Parts) -> Concluded<'file> {
        let merged = parts.into_iter().fold(Doubles::default(), |mut merged, part| {
            merged.declared.extend(part.declared);
            merged.implemented.extend(part.implemented);
            merged
        });
        merged.violations().into_iter().collect()
    }
}
