mod declaration;
mod declarations;
mod finding;
mod kind;
mod known;

use std::borrow::Cow;

use syn::spanned::Spanned;
use syn::{ItemImpl, ItemMod, Macro};

use lint_core::{Headline, Parsed, Site, SourceFile, Syntax, Violation};
use lint_file::{ContextLint, Reads};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;
use lint_sweep::{Descend, Finds, Judgement, Kind as Node, Watch};

pub use declaration::Declaration;
use declarations::Declarations;
use finding::Finding;
use kind::Kind;
pub use known::Known;

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

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        known: &'run Known<'head>,
    ) -> Judgement<'file, 'run> {
        Judgement::watching(Box::new(Predicates::new(known, file)))
    }
}

#[derive(Debug)]
pub struct Predicates<'scan, 'file> {
    known: &'scan Known<'scan>,
    outer: Vec<Cow<'file, ModulePath>>,
    file: &'file SourceFile,
    module: Cow<'file, ModulePath>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> Predicates<'scan, 'file> {
    pub fn new(known: &'scan Known<'scan>, file: &'file SourceFile) -> Predicates<'scan, 'file> {
        Predicates {
            known,
            outer: Vec::new(),
            file,
            module: Cow::Borrowed(file.head().placement().module()),
            violations: Vec::new(),
        }
    }

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
                    Site::new(self.file.head().path(), LineNumber::from(signature.span())),
                    Finding::BoolPredicate { name: Name::from(&signature.ident) },
                )
            })
            .collect()
    }
}

impl<'ast> Watch<'ast> for Predicates<'_, '_> {
    fn kinds(&self) -> &'static [Node] {
        &[Node::ItemMod, Node::ItemImpl, Node::Macro]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        let inner = Cow::Owned(self.module.child(Name::from(&module.ident)));
        self.outer.push(std::mem::replace(&mut self.module, inner));
        Descend::Into
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.module = self.outer.pop().unwrap_or_else(|| Cow::Borrowed(self.file.head().placement().module()));
    }

    fn on_item_impl(&mut self, block: &'ast ItemImpl) -> Descend {
        let found = Syntax::inherent(block)
            .filter(|candidate| self.known.kind(self.file, &self.module, &candidate.self_ty) == Kind::Enum)
            .map_or_default(|enum_impl| self.predicates(enum_impl));
        self.violations.extend(found);
        Descend::Into
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }
}

impl<'file> Finds<'file> for Predicates<'_, 'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}
