use std::path::Path;
mod declaration;
mod declarations;
mod finding;
mod kind;
mod known;

use syntax::Spanned;
use syntax::{ItemImpl, ItemMod, Macro, Type};

use lint_core::{Headline, Parsed, Site, SourceFile, Syntax, Violation};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;
use lint_sweep::{
    Contribute, Contributed, Contributing, Contribution, Descend, Facts, Gathers, Kind as Node, Scope, Sweep, Watching,
};

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
    file: &'file SourceFile,
    scan: Contributing<PredicateScan<'file>>,
}

#[derive(Debug)]
pub struct PredicateScan<'file> {
    file: &'file SourceFile,
}

#[derive(Debug)]
pub struct Unresolved<'file> {
    path: Vec<Name>,
    implemented: Type,
    violations: Vec<Violation<'file>>,
}

impl<'file> Unresolved<'file> {
    fn under(self, name: Option<&Name>) -> Unresolved<'file> {
        Unresolved { path: name.cloned().into_iter().chain(self.path).collect(), ..self }
    }

    fn candidate(self, module: &ModulePath) -> Option<Candidate<'file>> {
        let placed = self.path.into_iter().fold(module.clone(), |within, name| within.child(name));
        Target::implemented(&placed, &self.implemented).map(|target| Candidate { target, violations: self.violations })
    }
}

impl<'file> PredicateScan<'file> {
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

impl<'file> Contribute for PredicateScan<'file> {
    type Open = ();
    type Part = Vec<Unresolved<'file>>;

    fn kinds(&self) -> &'static [Node] {
        &[Node::ItemImpl, Node::Macro]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Node::ItemMod)] }
    }

    fn off_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        inner: Contribution<(), Vec<Unresolved<'file>>>,
    ) -> Vec<Unresolved<'file>> {
        inner.concluded(|(), pending| pending.into_iter().map(|within| within.under(facts.name())).collect())
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Vec<Unresolved<'file>>> {
        let found = facts.inherent().and_then(|block| {
            Some(self.predicates(block))
                .filter(|found| found.is_empty().then_some(()).is_none())
                .map(|violations| Unresolved { path: Vec::new(), implemented: Type::clone(&block.self_ty), violations })
        });
        Contributed::parted(found.into_iter().collect())
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Vec<Unresolved<'file>>> {
        Contributed::descending(Descend::Expand)
    }
}

impl<'file> Gathers<Candidates<'file>> for Predicates<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(&mut self.scan)
    }

    fn gathered(self) -> Candidates<'file> {
        let module = self.file.head().placement().module();
        Candidates(
            self.scan
                .concluded(|_, (), pending| pending.into_iter().filter_map(|found| found.candidate(module)).collect()),
        )
    }
}

impl<'file> From<&'file SourceFile> for Predicates<'file> {
    fn from(file: &'file SourceFile) -> Predicates<'file> {
        Predicates { file, scan: Contributing::new(PredicateScan { file }) }
    }
}
