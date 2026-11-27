use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use syn::visit::{self, Visit};
use syn::{Expr, ExprLit, ImplItemFn, ItemFn, ItemImpl, ItemMacro, ItemMod, Lit, Macro, Meta};

use crate::expansion::Expansion;
use crate::finding::{Finding, Fix, Found, Line, Rule};
use crate::gate::{Gate, TestMark};
use crate::graph::Graph;
use crate::name::Name;
use crate::source::{Place, Source};
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum ApiFact {
    Found(Finding),
    Emitter(Name),
    Invoked { name: Name, path: Arc<Path>, line: Line },
}

#[derive(Debug)]
pub(crate) struct PublicApiTests<'file> {
    source: &'file Source,
    facts: Vec<ApiFact>,
    inline: Vec<Name>,
}

#[derive(Debug, Default)]
struct Emitters(Vec<ApiFact>);

impl PublicApiTests<'_> {
    const IN_SOURCE: &'static str = "test code in src";
    const IN_SOURCE_FIX: Fix = Fix("move it to the package's tests directory and call the public API from there");
    const PATH_ATTRIBUTE: &'static str = "path";
    const SOURCE_DIRECTORY: &'static str = "src";

    fn in_source(&mut self, line: Line) {
        let found = Found::at(line, PublicApiTests::IN_SOURCE, PublicApiTests::IN_SOURCE_FIX);
        self.facts.push(ApiFact::Found(found.in_file(Rule::PublicApiTests, self.source.shown())));
    }

    fn shipped(&self) -> Option<()> {
        (self.source.place() == Place::Shipped).then_some(())
    }

    fn skipped(&mut self, line: Line, walk: impl FnOnce(&mut Emitters)) {
        self.in_source(line);
        let mut emitters = Emitters::default();
        walk(&mut emitters);
        self.facts.extend(emitters.0);
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
        let within = self.inline.iter().fold(PathBuf::new(), |inside, name| inside.join(&**name));
        let reaching =
            target.filter(|path| PublicApiTests::lexical(&directory.join(&within).join(path)).starts_with(&source));
        self.facts.extend(reaching.map(|_| {
            let found = Found::spanned(
                module.ident.span(),
                "test file includes a module from src with #[path]",
                Fix("call the public API instead"),
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
    fn defined(definition: &ItemMacro) -> Option<ApiFact> {
        let name = definition.ident.as_ref()?;
        Expansion::defining(&definition.mac)?;
        let tested = TestMark::written(definition.mac.tokens.clone()).iter().find_map(TestMark::on).map(|_| ());
        tested.map(|()| ApiFact::Emitter(Name::of(name)))
    }
}

impl<'ast> Visit<'ast> for Emitters {
    fn visit_item_macro(&mut self, definition: &'ast ItemMacro) {
        self.0.extend(Emitters::defined(definition));
        visit::visit_item_macro(self, definition);
    }
}

impl<'ast> Visit<'ast> for PublicApiTests<'_> {
    fn visit_item_macro(&mut self, definition: &'ast ItemMacro) {
        self.facts.extend(Emitters::defined(definition));
        visit::visit_item_macro(self, definition);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let line = Line::of(module.ident.span());
        let skipped = self.shipped().and_then(|()| Gate::of(&module.attrs).entered().xor(Some(())));
        if skipped.is_some() {
            self.skipped(line, |emitters| emitters.visit_item_mod(module));
        } else {
            self.shipped().xor(Some(())).map_or_default(|()| self.reached(module));
            self.inline.push(Name::of(&module.ident));
            visit::visit_item_mod(self, module);
            self.inline.pop();
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let skipped = self.shipped().and_then(|()| Gate::of(&item.attrs).entered().xor(Some(())));
        match skipped {
            Some(()) => self.skipped(Line::of(item.impl_token.span), |emitters| emitters.visit_item_impl(item)),
            None => visit::visit_item_impl(self, item),
        }
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        let skipped = self.shipped().and_then(|()| Gate::of(&function.attrs).entered().xor(Some(())));
        match skipped {
            Some(()) => self.skipped(Line::of(function.sig.ident.span()), |emitters| {
                emitters.visit_impl_item_fn(function);
            }),
            None => visit::visit_impl_item_fn(self, function),
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let line = Line::of(function.sig.ident.span());
        let skipped = self.shipped().and_then(|()| Gate::of(&function.attrs).entered().xor(Some(())));
        if skipped.is_some() {
            self.skipped(line, |emitters| emitters.visit_item_fn(function));
        } else {
            let marked = self.shipped().and(TestMark::among(&function.attrs));
            marked.map_or_default(|_| self.in_source(line));
            visit::visit_item_fn(self, function);
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let called = self.shipped().and(Expansion::defining(invocation).xor(Some(())));
        let line = Line::of(invocation.bang_token.span);
        let marked = called
            .and_then(|()| TestMark::written(invocation.tokens.clone()).iter().find_map(TestMark::on).map(|_| ()));
        let invoked = called.and(invocation.path.get_ident()).map(|name| ApiFact::Invoked {
            name: Name::of(name),
            path: Arc::clone(self.source.shown()),
            line,
        });
        match marked {
            Some(()) => self.in_source(line),
            None => self.facts.extend(invoked),
        }
    }
}

impl<'file> Graph<'file> for PublicApiTests<'file> {
    const RULE: Rule = Rule::PublicApiTests;

    type Fact = ApiFact;

    fn new(source: &'file Source) -> PublicApiTests<'file> {
        PublicApiTests { source, facts: Vec::new(), inline: Vec::new() }
    }

    fn facts(self) -> Vec<ApiFact> {
        self.facts
    }

    fn reduce(facts: Vec<ApiFact>, _: &[Arc<Package>]) -> Vec<Finding> {
        let emitters: BTreeSet<Name> = facts
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
