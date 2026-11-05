use std::collections::BTreeSet;
use std::iter;
use std::ops::RangeInclusive;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemForeignMod, ItemImpl, ItemMod};

use lint_core::{Site, SourceFile, Walk};
use lint_effect_sites::EffectSites;
use lint_fakes_at_ports::{Kind, PortTraits};
use lint_layout::{LineNumber, UsePath};
use lint_paths::{Imports, Module, Nested};

#[derive(Debug)]
pub struct Adapters<'file> {
    places: BTreeSet<Site<'file>>,
    modules: BTreeSet<Within<'file>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Within<'file> {
    path: &'file Path,
    first: LineNumber,
    last: LineNumber,
}

impl<'file> Within<'file> {
    fn new(path: &'file Path, first: LineNumber, last: LineNumber) -> Within<'file> {
        Within { path, first, last }
    }

    fn span(self) -> RangeInclusive<LineNumber> {
        self.first..=self.last
    }
}

impl<'file> From<&'file [SourceFile]> for Adapters<'file> {
    fn from(files: &'file [SourceFile]) -> Adapters<'file> {
        let every: Vec<&SourceFile> =
            files.iter().flat_map(|file| iter::once(file).chain(file.test_modules())).collect();
        let imports = Imports::across(every.iter().copied());
        let ports = PortTraits::declared(files);
        let effects = Adapters { places: EffectSites::places(files), modules: BTreeSet::new() };
        every.iter().map(|file| Reader::read(file, &imports, &ports)).fold(effects, Adapters::merged)
    }
}

impl<'file> Adapters<'file> {
    fn merged(mut self, other: Adapters<'file>) -> Adapters<'file> {
        self.places.extend(other.places);
        self.modules.extend(other.modules);
        self
    }

    pub fn evidence(&self, path: &'file Path, first: LineNumber, last: LineNumber) -> Option<&Site<'file>> {
        let whole = Within::new(path, first, last);
        let inner: Vec<RangeInclusive<LineNumber>> = self
            .modules
            .range(Within::new(path, first, first)..=Within::new(path, last, last))
            .filter(|nested| **nested != whole)
            .map(|nested| nested.span())
            .collect();
        self.places
            .range(Site::new(path, first)..=Site::new(path, last))
            .find(|site| inner.iter().find(|span| span.contains(&site.line())).is_none())
    }
}

struct Reader<'file, 'run> {
    file: &'file SourceFile,
    module: Module<'file>,
    imports: &'run Imports<'run>,
    ports: &'run PortTraits,
    found: Adapters<'file>,
}

impl<'file, 'run> Reader<'file, 'run> {
    fn read(file: &'file SourceFile, imports: &'run Imports<'run>, ports: &'run PortTraits) -> Adapters<'file> {
        let found = Adapters { places: BTreeSet::new(), modules: BTreeSet::new() };
        let mut reader = Reader { file, module: Module::of_file(file), imports, ports, found };
        reader.visit_file(file.syntax());
        reader.found
    }

    fn implements_port(&self, item: &ItemImpl) -> Option<Site<'file>> {
        item.trait_.as_ref().and_then(|(_, path, _)| {
            let written = UsePath::from(path);
            let bare = Kind::written(&written);
            let candidates = self.imports.candidates(&self.module, written);
            Some(bare.judged(&candidates, self.ports))
                .filter(|kind| *kind == Kind::Port)
                .map(|_| Site::new(self.file.path(), LineNumber::from(item.span())))
        })
    }
}

impl<'file> Nested<'file> for Reader<'file, '_> {
    fn module(&mut self) -> &mut Module<'file> {
        &mut self.module
    }
}

impl<'file> Visit<'file> for Reader<'file, '_> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        let span = module.span();
        self.found.modules.insert(Within::new(
            self.file.path(),
            LineNumber::from(span.start().line),
            LineNumber::from(span.end().line),
        ));
        self.nested(module, |reader| visit::visit_item_mod(reader, module));
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        Walk::enter(&item.attrs, || {
            self.found.places.extend(self.implements_port(item));
            visit::visit_item_impl(self, item);
        });
    }

    fn visit_item_foreign_mod(&mut self, item: &'file ItemForeignMod) {
        Walk::enter(&item.attrs, || {
            self.found.places.insert(Site::new(self.file.path(), LineNumber::from(item.span())));
        });
    }
}
