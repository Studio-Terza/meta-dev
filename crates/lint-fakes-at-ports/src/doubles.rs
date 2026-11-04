mod finding;

use std::collections::BTreeMap;
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemUnion, Path as TraitPath};

use lint_core::{Site, SourceFile, Violation, Walk};
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
    trait_path: &'file TraitPath,
    kind: Kind,
}

impl<'file> Doubles<'file> {
    pub fn scan(&mut self, file: &'file SourceFile, imports: &Imports<'_>, ports: &PortTraits) {
        let mut reader = Reader {
            file,
            target: file.placement().crate_root().directory(),
            module: Module::of_file(file),
            imports,
            ports,
            doubles: self,
        };
        reader.visit_file(file.syntax());
    }

    pub fn violations(self) -> Vec<Violation<'file>> {
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
            Violation::new(
                implemented.site,
                Finding::Beside(&implemented.double.name, UsePath::from(implemented.trait_path)),
            )
        });
        let mut violations: Vec<Violation<'file>> = unported.chain(beside).collect();
        violations.sort_by(|left, right| left.site().cmp(right.site()));
        violations
    }
}

struct Reader<'file, 'run> {
    file: &'file SourceFile,
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
        let site = Site::new(self.file.path(), LineNumber::from(name.span()));
        self.doubles.declared.extend(double.name.matches::<CamelWord>(&Self::DOUBLES).map(|_| (double, site)));
    }

    fn double_of(&self, item: &ItemImpl) -> Option<DoubleKey<'file>> {
        Name::of_type(&item.self_ty)
            .filter(|name| name.matches::<CamelWord>(&Self::DOUBLES).is_some())
            .map(|name| DoubleKey { target: self.target, name })
    }

    fn implementation(&self, double: DoubleKey<'file>, path: &'file TraitPath) -> Implemented<'file> {
        let written = UsePath::from(path);
        let bare = Kind::written(&written);
        let candidates = self.imports.candidates(&self.module, written);
        let line = path.segments.first().map_or(LineNumber::FIRST, |first| LineNumber::from(first.ident.span()));
        Implemented {
            double,
            site: Site::new(self.file.path(), line),
            kind: bare.judged(&candidates, self.ports),
            trait_path: path,
        }
    }
}

impl<'file> Nested<'file> for Reader<'file, '_> {
    fn module(&mut self) -> &mut Module<'file> {
        &mut self.module
    }
}

impl<'file> Visit<'file> for Reader<'file, '_> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        self.nested(module, |reader| visit::visit_item_mod(reader, module));
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        Walk::enter(&item.attrs, || {
            let found = self
                .double_of(item)
                .zip(item.trait_.as_ref())
                .map(|(double, (_, path, _))| self.implementation(double, path));
            self.doubles.implemented.extend(found);
        });
    }

    fn visit_item_struct(&mut self, item: &'file ItemStruct) {
        Walk::enter(&item.attrs, || self.declared(&item.ident));
    }

    fn visit_item_enum(&mut self, item: &'file ItemEnum) {
        Walk::enter(&item.attrs, || self.declared(&item.ident));
    }

    fn visit_item_union(&mut self, item: &'file ItemUnion) {
        Walk::enter(&item.attrs, || self.declared(&item.ident));
    }
}
