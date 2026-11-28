use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pipeline::Measure;
use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMacro, ItemMod, Macro};

use crate::expansion::Expansion;
use crate::finding::{Finding, Found};
use crate::gate::{Gate, TestMark};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Fact, Lint, Reads};
use crate::source::{Place, Source};
use crate::tree::Tree;
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum ApiFact {
    Found(Finding),
    Emitter(String),
    Invoked { name: String, path: Arc<Path>, line: usize },
}

#[derive(Debug, Measure)]
pub(crate) struct ApiFacts(Vec<ApiFact>);

#[derive(Debug)]
pub(crate) struct PublicApiTests<'file> {
    source: &'file Source,
    facts: Vec<ApiFact>,
    inline: Vec<String>,
    hidden: usize,
}

impl PublicApiTests<'_> {
    const IN_SOURCE: &'static str = "test code in src";
    const IN_SOURCE_FIX: &'static str = "move it to the package's tests directory and call the public API from there";
    const SOURCE_DIRECTORY: &'static str = "src";

    fn in_source(&mut self, line: usize) {
        let fact = self.found_in_source(line);
        self.facts.push(fact);
    }

    fn found_in_source(&self, line: usize) -> ApiFact {
        let found = Found::at(line, PublicApiTests::IN_SOURCE, PublicApiTests::IN_SOURCE_FIX);
        ApiFact::Found(found.in_file(PublicApiTests::HEADLINE, self.source.shown()))
    }

    fn shipped(&self) -> Option<()> {
        (self.source.place() == Place::Shipped).then_some(())
    }

    fn seen(&self) -> Option<()> {
        (self.hidden == 0).then_some(())
    }

    fn walked(&mut self, attributes: &[Attribute], line: usize) -> Option<()> {
        let skipped = self.shipped().and_then(|()| Gate::of(attributes).entered().xor(Some(())));
        skipped.map_or_default(|()| self.in_source(line));
        skipped.xor(Some(()))
    }

    fn entered(&mut self, attributes: &[Attribute], line: usize) -> Option<()> {
        let walked = self.seen().and_then(|()| self.walked(attributes, line));
        self.hidden += usize::from(walked.is_none());
        walked
    }

    fn left(&mut self) -> Option<()> {
        let shown = self.seen();
        self.hidden = self.hidden.saturating_sub(1);
        shown
    }

    fn reached(&mut self, module: &ItemMod) {
        let directory = self.source.real().parent().map_or_default(Path::to_path_buf);
        let source = Tree::lexical(&self.source.package().directory().join(PublicApiTests::SOURCE_DIRECTORY));
        let within = self.inline.iter().fold(PathBuf::new(), |inside, name| inside.join(name));
        let reaching = Tree::path_attribute(&module.attrs)
            .filter(|path| Tree::lexical(&directory.join(&within).join(path)).starts_with(&source));
        self.facts.extend(reaching.map(|_| {
            let found = Found::spanned(
                module.ident.span(),
                "test file includes a module from src with #[path]",
                "call the public API instead",
            );
            ApiFact::Found(found.in_file(PublicApiTests::HEADLINE, self.source.shown()))
        }));
    }

    fn emitter(definition: &ItemMacro) -> Option<ApiFact> {
        let name = definition.ident.as_ref()?;
        Expansion::defining(&definition.mac)?;
        let tested = TestMark::written(definition.mac.tokens.clone()).iter().find_map(TestMark::on).map(|_| ());
        tested.map(|()| ApiFact::Emitter(name.to_string()))
    }
}

impl Hooks for PublicApiTests<'_> {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        let walked = self.entered(&module.attrs, module.ident.span().start().line);
        walked.map_or_default(|()| {
            self.shipped().xor(Some(())).map_or_default(|()| self.reached(module));
            self.inline.push(module.ident.to_string());
        });
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.left().map_or_default(|()| {
            self.inline.pop();
        });
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        self.entered(&item.attrs, item.impl_token.span.start().line);
        Descent::Into
    }

    fn left_item_impl(&mut self, _: &ItemImpl) {
        self.left();
    }

    fn impl_item_fn(&mut self, function: &ImplItemFn) -> Descent {
        self.entered(&function.attrs, function.sig.ident.span().start().line);
        Descent::Into
    }

    fn left_impl_item_fn(&mut self, _: &ImplItemFn) {
        self.left();
    }

    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        let line = function.sig.ident.span().start().line;
        let walked = self.entered(&function.attrs, line);
        let marked = walked.and(self.shipped()).and(TestMark::among(&function.attrs));
        marked.map_or_default(|_| self.in_source(line));
        Descent::Into
    }

    fn left_item_fn(&mut self, _: &ItemFn) {
        self.left();
    }

    fn item_macro(&mut self, definition: &ItemMacro) -> Descent {
        self.facts.extend(PublicApiTests::emitter(definition));
        Descent::Into
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        let called = self.seen().and(self.shipped()).and(Expansion::defining(invocation).xor(Some(())));
        let line = invocation.bang_token.span.start().line;
        let marked = called
            .and_then(|()| TestMark::written(invocation.tokens.clone()).iter().find_map(TestMark::on).map(|_| ()));
        let invoked = called.and(invocation.path.get_ident()).map(|name| ApiFact::Invoked {
            name: name.to_string(),
            path: Arc::clone(self.source.shown()),
            line,
        });
        let fact = marked.map_or(invoked, |()| Some(self.found_in_source(line)));
        self.facts.extend(fact);
        Expanded::Nothing
    }
}

impl<'file> Lint<'file> for PublicApiTests<'file> {
    const HEADLINE: &'static str = "tests go through public APIs";
    const READS: Reads = Reads::Every;

    type Fact = ApiFacts;

    fn new(source: &'file Source) -> PublicApiTests<'file> {
        PublicApiTests { source, facts: Vec::new(), inline: Vec::new(), hidden: 0 }
    }

    fn fact(self, _: &'file Source) -> ApiFacts {
        ApiFacts(self.facts)
    }
}

impl Fact for ApiFacts {
    fn findings(self, _: &[Arc<Package>]) -> Vec<Finding> {
        let emitters: BTreeSet<String> = self
            .0
            .iter()
            .filter_map(|fact| match fact {
                ApiFact::Emitter(name) => Some(name.clone()),
                ApiFact::Found(_) | ApiFact::Invoked { .. } => None,
            })
            .collect();
        self.0
            .into_iter()
            .filter_map(|fact| match fact {
                ApiFact::Found(finding) => Some(finding),
                ApiFact::Invoked { name, path, line } => emitters.contains(&name).then(|| {
                    Found::at(line, PublicApiTests::IN_SOURCE, PublicApiTests::IN_SOURCE_FIX)
                        .in_file(PublicApiTests::HEADLINE, &path)
                }),
                ApiFact::Emitter(_) => None,
            })
            .collect()
    }
}
