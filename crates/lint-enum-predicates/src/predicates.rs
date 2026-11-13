mod declaration;
mod declarations;
mod finding;
mod kind;
mod known;

use std::borrow::Cow;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemImpl, ItemMod, Macro};

use lint_core::{Headline, Macros, Parsed, Site, SourceFile, Syntax, Violation};
use lint_file::{ContextLint, Reads};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;

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

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, known: &Known<'head>) -> Vec<Violation<'file>> {
        let mut scan = Predicates::new(known, file);
        scan.visit_file(file.syntax());
        scan.into_violations()
    }
}

pub struct Predicates<'scan, 'file> {
    known: &'scan Known<'scan>,
    file: &'file SourceFile,
    module: Cow<'file, ModulePath>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> Predicates<'scan, 'file> {
    pub fn new(known: &'scan Known<'scan>, file: &'file SourceFile) -> Predicates<'scan, 'file> {
        Predicates { known, file, module: Cow::Borrowed(file.head().placement().module()), violations: Vec::new() }
    }

    pub fn into_violations(self) -> Vec<Violation<'file>> {
        self.violations
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

impl<'ast> Visit<'ast> for Predicates<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inner = Cow::Owned(self.module.child(Name::from(&module.ident)));
        let outer = std::mem::replace(&mut self.module, inner);
        syn::visit::visit_item_mod(self, module);
        self.module = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let found = Syntax::inherent(block)
            .filter(|candidate| self.known.kind(self.file, &self.module, &candidate.self_ty) == Kind::Enum)
            .map_or_default(|enum_impl| self.predicates(enum_impl));
        self.violations.extend(found);
        syn::visit::visit_item_impl(self, block);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }
}
