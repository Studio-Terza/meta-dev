mod finding;
mod kind;
mod words;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemUnion, Path as TraitPath, Type};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name, UsePath};
use lint_paths::{Imports, Module};
use lint_placement::Role;

use crate::port_traits::PortTraits;
use finding::Finding;
pub use kind::Kind;
use words::DoubleWords;

#[derive(Debug, Default)]
pub struct Doubles {
    declared: BTreeMap<DoubleKey, Site>,
    implemented: Vec<Implemented>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DoubleKey {
    target: PathBuf,
    name: Name,
}

#[derive(Debug)]
struct Implemented {
    double: DoubleKey,
    site: Site,
    trait_path: UsePath,
    kind: Kind,
}

impl Doubles {
    const TESTS: &str = "tests";

    pub fn tested(file: &SourceFile) -> Option<&SourceFile> {
        let placement = file.placement();
        (placement.role() == Role::Target
            && placement.crate_root().directory().parent().and_then(Path::file_name) == Some(OsStr::new(Self::TESTS)))
        .then_some(file)
    }

    pub fn scan(&mut self, file: &SourceFile, imports: &Imports, ports: &PortTraits) {
        let mut reader = Reader {
            file,
            target: file.placement().crate_root().directory().to_path_buf(),
            module: Module::of_file(file),
            imports,
            ports,
            doubles: self,
        };
        reader.visit_file(file.syntax());
    }

    pub fn violations(self) -> Vec<Violation> {
        let ported = |double: &DoubleKey| {
            self.implemented.iter().any(|implemented| implemented.double == *double && implemented.kind == Kind::Port)
        };
        let unported = self
            .declared
            .iter()
            .filter(|(double, _)| !ported(double))
            .map(|(double, site)| Violation::new(site.clone(), Finding::Unported(double.name.clone())));
        let beside = self.implemented.iter().filter(|implemented| implemented.kind == Kind::Other).map(|implemented| {
            Violation::new(
                implemented.site.clone(),
                Finding::Beside(implemented.double.name.clone(), implemented.trait_path.clone()),
            )
        });
        let mut violations: Vec<Violation> = unported.chain(beside).collect();
        violations.sort_by(|left, right| left.site().cmp(right.site()));
        violations
    }
}

struct Reader<'run> {
    file: &'run SourceFile,
    target: PathBuf,
    module: Module,
    imports: &'run Imports,
    ports: &'run PortTraits,
    doubles: &'run mut Doubles,
}

impl Reader<'_> {
    fn declared(&mut self, name: &Ident) {
        let double = DoubleKey { target: self.target.clone(), name: Name::from(name) };
        let site = Site::new(self.file.path(), LineNumber::from(name.span()));
        self.doubles.declared.extend(DoubleWords::name(&double.name).map(|_| (double, site)));
    }

    fn double_of(&self, item: &ItemImpl) -> Option<DoubleKey> {
        let name = match item.self_ty.as_ref() {
            Type::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        };
        name.filter(|name| DoubleWords::name(name).is_some())
            .map(|name| DoubleKey { target: self.target.clone(), name })
    }

    fn implementation(&self, double: DoubleKey, path: &TraitPath) -> Implemented {
        let trait_path = UsePath::from(path);
        let candidates = self.imports.candidates(&self.module, &trait_path);
        let line = path.segments.first().map_or(LineNumber::FIRST, |first| LineNumber::from(first.ident.span()));
        Implemented {
            double,
            site: Site::new(self.file.path(), line),
            kind: Kind::judged(&trait_path, &candidates, self.ports),
            trait_path,
        }
    }
}

impl<'ast> Visit<'ast> for Reader<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.module.clone();
        self.module = outer.within(&Name::from(&module.ident));
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.module = outer;
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
