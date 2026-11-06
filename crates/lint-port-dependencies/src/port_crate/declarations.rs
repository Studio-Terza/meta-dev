use std::collections::BTreeSet;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, ItemImpl, ItemTrait};

use lint_core::{Site, SourceFile};
use lint_layout::{LineNumber, Name, UsePath, Whole, Word, words};

pub(super) struct Declarations<'file, 'ports> {
    file: &'file SourceFile,
    ports: &'ports BTreeSet<Name>,
    declared: BTreeSet<Name>,
    fillers: BTreeSet<Site<'file>>,
}

impl<'file, 'ports> Declarations<'file, 'ports> {
    const PORT: [Word<'static>; 1] = words!["port"];
    const QUALIFIED: [Word<'static>; 2] = words!["port_core", "port"];
    const FILLER: Word<'static> = Word::new("Filler");
    const RELEASE: [Word<'static>; 1] = words!["Release"];

    pub(super) fn ports(file: &'file SourceFile) -> BTreeSet<Name> {
        Declarations::walked(file, &BTreeSet::new()).declared
    }

    pub(super) fn fillers(file: &'file SourceFile, ports: &'ports BTreeSet<Name>) -> BTreeSet<Site<'file>> {
        Declarations::walked(file, ports).fillers
    }

    fn walked(file: &'file SourceFile, ports: &'ports BTreeSet<Name>) -> Declarations<'file, 'ports> {
        let mut walk = Declarations { file, ports, declared: BTreeSet::new(), fillers: BTreeSet::new() };
        walk.visit_file(file.syntax());
        walk
    }

    fn ported(attribute: &Attribute) -> Option<UsePath> {
        let path = UsePath::from(attribute.path());
        path.spells(&Self::PORT).or_else(|| path.spells(&Self::QUALIFIED)).cloned()
    }

    fn filled(&self, block: &ItemImpl) -> Option<Span> {
        let last = block.trait_.as_ref().and_then(|(_, path, _)| path.segments.last())?;
        let filler = self.ports.iter().find(|port| port.suffixed(Self::FILLER) == last.ident).map(|_| ());
        let release = Name::from(&last.ident).matches::<Whole>(&Self::RELEASE).map(|_| ());
        filler.or(release).map(|()| block.span())
    }
}

impl<'ast> Visit<'ast> for Declarations<'_, '_> {
    fn visit_item_trait(&mut self, declared: &'ast ItemTrait) {
        let ported = declared.attrs.iter().find_map(Self::ported).map(|_| Name::from(&declared.ident));
        self.declared.extend(ported);
        visit::visit_item_trait(self, declared);
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let lines = self.filled(block).map(|span| span.start().line..=span.end().line).into_iter().flatten();
        let file = self.file.path();
        self.fillers.extend(lines.map(|line| Site::new(file, LineNumber::from(line))));
        visit::visit_item_impl(self, block);
    }
}
