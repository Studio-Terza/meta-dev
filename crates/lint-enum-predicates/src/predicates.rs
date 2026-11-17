use std::path::Path;
mod declaration;
mod declarations;
mod finding;
mod kind;
mod known;

use std::borrow::Cow;

use syntax::Spanned;
use syntax::{ItemImpl, ItemMod, Macro};

use lint_core::{Headline, Parsed, Site, SourceFile, Syntax, Violation};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;
use lint_sweep::{Descend, Facts, Gathers, Kind as Node, Watch, Watching};

pub use declaration::Declaration;
use declarations::Declarations;
use finding::Finding;
use kind::Kind;
pub use known::{Known, Target};

#[derive(Debug)]
pub struct EnumPredicates;

impl ContextLint for EnumPredicates {
    const HEADLINE: Headline = Headline::new(
        "an enum method returning bool throws away the variant; return an enum or match where the answer is used",
    );
    const READS: Reads = Reads::Every;

    type Part<'head> = Vec<Declaration<'head>>;
    type Context<'head> = Known<'head>;

    type Gathering<'head> = Declarations<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> Declarations<'head> {
        Declarations::in_file(parsed)
    }
}

impl WatchingLint for EnumPredicates {
    type Found<'file> = Candidates<'file>;
    type Watched<'file> = Predicates<'file>;

    fn watched(file: &SourceFile) -> Predicates<'_> {
        Predicates::from(file)
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        found: Candidates<'file>,
        known: &Known<'head>,
    ) -> Vec<Violation<'file>> {
        let enums = found.0.into_iter().filter(|candidate| known.kind(file, &candidate.target) == Kind::Enum);
        enums.flat_map(|candidate| candidate.violations).collect()
    }
}

#[derive(Debug)]
pub struct Candidate<'file> {
    target: Target,
    violations: Vec<Violation<'file>>,
}

#[derive(Debug)]
pub struct Candidates<'file>(Vec<Candidate<'file>>);

#[derive(Debug)]
pub struct Predicates<'file> {
    candidates: Vec<Candidate<'file>>,
    outer: Vec<Cow<'file, ModulePath>>,
    file: &'file SourceFile,
    module: Cow<'file, ModulePath>,
}

impl<'file> Predicates<'file> {
    fn predicates(&self, block: &ItemImpl) -> Vec<Violation<'file>> {
        block
            .items
            .iter()
            .filter_map(Syntax::impl_fn)
            .map(|function| &function.sig)
            .filter(|signature| {
                Syntax::returned(&signature.output)
                    .and_then(Syntax::type_path)
                    .is_some_and(|path| path.path.is_ident("bool"))
            })
            .map(|signature| {
                Violation::new(
                    Site::new(AsRef::<Path>::as_ref(self.file.head()), LineNumber::from(signature.span())),
                    Finding::BoolPredicate { name: Name::from(&signature.ident) },
                )
            })
            .collect()
    }
}

impl<'ast> Watch<'ast> for Predicates<'_> {
    fn kinds(&self) -> &'static [Node] {
        &[Node::ItemMod, Node::ItemImpl, Node::Macro]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        let inner =
            facts.name().map_or_else(|| self.module.clone(), |name| Cow::Owned(self.module.child(name.clone())));
        self.outer.push(std::mem::replace(&mut self.module, inner));
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod, _: &Facts<'_, ItemMod>) {
        self.module = self.outer.pop().unwrap_or_else(|| Cow::Borrowed(self.file.head().placement().module()));
    }

    fn on_item_impl(&mut self, _: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        let found = facts.inherent().and_then(|block| {
            let target = Target::implemented(&self.module, &block.self_ty)?;
            let violations = self.predicates(block);
            Some(violations)
                .filter(|found| found.is_empty().then_some(()).is_none())
                .map(|found| Candidate { target, violations: found })
        });
        self.candidates.extend(found);
        Descend::Into
    }

    fn on_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        Descend::Expand
    }
}

impl<'file> Gathers<Candidates<'file>> for Predicates<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(self)
    }

    fn gathered(self) -> Candidates<'file> {
        Candidates(self.candidates)
    }
}

impl<'file> From<&'file SourceFile> for Predicates<'file> {
    fn from(file: &'file SourceFile) -> Predicates<'file> {
        Predicates {
            candidates: Vec::new(),
            outer: Vec::new(),
            file,
            module: Cow::Borrowed(file.head().placement().module()),
        }
    }
}
