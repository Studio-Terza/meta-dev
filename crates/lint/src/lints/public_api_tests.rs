use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use syn::{Attribute, Expr, ExprLit, ImplItemFn, ItemFn, ItemImpl, ItemMacro, ItemMod, Lit, Macro, Meta};

use crate::expansion::Expansion;
use crate::finding::{Finding, Found, Rule};
use crate::gate::{Gate, TestMark};
use crate::graph::Graph;
use crate::source::{Place, Source};
use crate::walk::{Descent, Expanded, Hooks};
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum ApiFact {
    Found(Finding),
    Emitter(String),
    Invoked { name: String, path: Arc<Path>, line: usize },
}

#[derive(Debug)]
pub(crate) struct PublicApiTests<'file> {
    source: &'file Source,
    facts: Vec<ApiFact>,
    inline: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Emitters(Vec<ApiFact>);

impl PublicApiTests<'_> {
    const IN_SOURCE: &'static str = "test code in src";
    const IN_SOURCE_FIX: &'static str = "move it to the package's tests directory and call the public API from there";
    const PATH_ATTRIBUTE: &'static str = "path";
    const SOURCE_DIRECTORY: &'static str = "src";

    fn in_source(&mut self, line: usize) {
        let fact = self.found_in_source(line);
        self.facts.push(fact);
    }

    fn found_in_source(&self, line: usize) -> ApiFact {
        let found = Found::at(line, PublicApiTests::IN_SOURCE, PublicApiTests::IN_SOURCE_FIX);
        ApiFact::Found(found.in_file(Rule::PublicApiTests, self.source.shown()))
    }

    fn shipped(&self) -> Option<()> {
        (self.source.place() == Place::Shipped).then_some(())
    }

    fn walked(&mut self, attributes: &[Attribute], line: usize) -> Option<()> {
        let skipped = self.shipped().and_then(|()| Gate::of(attributes).entered().xor(Some(())));
        skipped.map_or_default(|()| self.in_source(line));
        skipped.xor(Some(()))
    }

    fn reached(&mut self, module: &ItemMod) {
        let target =
            module.attrs.iter().filter(|attribute| attribute.path().is_ident(PublicApiTests::PATH_ATTRIBUTE)).find_map(
                |attribute| match &attribute.meta {
                    Meta::NameValue(pair) => match &pair.value {
                        Expr::Lit(ExprLit { lit: Lit::Str(path), .. }) => Some(path.value()),
                        _ => None,
                    },
                    Meta::Path(_) | Meta::List(_) => None,
                },
            );
        let directory = self.source.real().parent().map_or_default(Path::to_path_buf);
        let source = PublicApiTests::lexical(&self.source.package().directory().join(PublicApiTests::SOURCE_DIRECTORY));
        let within = self.inline.iter().fold(PathBuf::new(), |inside, name| inside.join(name));
        let reaching =
            target.filter(|path| PublicApiTests::lexical(&directory.join(&within).join(path)).starts_with(&source));
        self.facts.extend(reaching.map(|_| {
            let found = Found::spanned(
                module.ident.span(),
                "test file includes a module from src with #[path]",
                "call the public API instead",
            );
            ApiFact::Found(found.in_file(Rule::PublicApiTests, self.source.shown()))
        }));
    }

    fn lexical(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |folded, component| match component {
            Component::ParentDir => match folded.file_name() {
                Some(_) => folded.parent().map_or_default(Path::to_path_buf),
                None => folded.join(component),
            },
            Component::CurDir => folded,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => folded.join(component),
        })
    }
}

impl Emitters {
    pub(crate) fn facts(self) -> Vec<ApiFact> {
        self.0
    }

    fn defined(definition: &ItemMacro) -> Option<ApiFact> {
        let name = definition.ident.as_ref()?;
        Expansion::defining(&definition.mac)?;
        let tested = TestMark::written(definition.mac.tokens.clone()).iter().find_map(TestMark::on).map(|_| ());
        tested.map(|()| ApiFact::Emitter(name.to_string()))
    }
}

impl Hooks for Emitters {
    fn item_macro(&mut self, definition: &ItemMacro) -> Descent {
        self.0.extend(Emitters::defined(definition));
        Descent::Into
    }
}

impl Hooks for PublicApiTests<'_> {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        let walked = self.walked(&module.attrs, module.ident.span().start().line);
        walked.map_or_default(|()| {
            self.shipped().xor(Some(())).map_or_default(|()| self.reached(module));
            self.inline.push(module.ident.to_string());
        });
        Descent::from(walked)
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.inline.pop();
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(self.walked(&item.attrs, item.impl_token.span.start().line))
    }

    fn impl_item_fn(&mut self, function: &ImplItemFn) -> Descent {
        Descent::from(self.walked(&function.attrs, function.sig.ident.span().start().line))
    }

    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        let line = function.sig.ident.span().start().line;
        let walked = self.walked(&function.attrs, line);
        let marked = walked.and(self.shipped()).and(TestMark::among(&function.attrs));
        marked.map_or_default(|_| self.in_source(line));
        Descent::from(walked)
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        let called = self.shipped().and(Expansion::defining(invocation).xor(Some(())));
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

impl<'file> Graph<'file> for PublicApiTests<'file> {
    type Fact = ApiFact;

    fn new(source: &'file Source) -> PublicApiTests<'file> {
        PublicApiTests { source, facts: Vec::new(), inline: Vec::new() }
    }

    fn facts(self) -> Vec<ApiFact> {
        self.facts
    }

    fn reduce(facts: Vec<ApiFact>, _: &[Arc<Package>]) -> Vec<Finding> {
        let emitters: BTreeSet<String> = facts
            .iter()
            .filter_map(|fact| match fact {
                ApiFact::Emitter(name) => Some(name.clone()),
                ApiFact::Found(_) | ApiFact::Invoked { .. } => None,
            })
            .collect();
        facts
            .into_iter()
            .filter_map(|fact| match fact {
                ApiFact::Found(finding) => Some(finding),
                ApiFact::Invoked { name, path, line } => emitters.contains(&name).then(|| {
                    Found::at(line, PublicApiTests::IN_SOURCE, PublicApiTests::IN_SOURCE_FIX)
                        .in_file(Rule::PublicApiTests, &path)
                }),
                ApiFact::Emitter(_) => None,
            })
            .collect()
    }
}
