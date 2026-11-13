mod declaration;
mod declarations;
mod finding;
mod kind;
mod known;

use std::borrow::Cow;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemImpl, ItemMod, Macro, Type, TypePath};

use lint_core::{Headline, Lint, Macros, Report, Site, SourceFile, Syntax, Violation};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;

use declaration::Declaration;
use declarations::Declarations;
use finding::Finding;
use kind::Kind;
use known::Known;

#[derive(Debug, Default)]
pub struct EnumPredicates<'file> {
    known: Known<'file>,
}

impl Lint for EnumPredicates<'_> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let declarations: Vec<Declaration<'_>> = files.iter().flat_map(Declarations::in_file).collect();
        let predicates =
            EnumPredicates { known: declarations.iter().fold(Known::default(), |known, found| found.filed(known)) };
        Report::new(
            Headline::new(
                "an enum method returning bool throws away the variant; return an enum or match where the answer is used",
            ),
            files.iter().flat_map(|file| predicates.violations(file)).collect(),
        )
    }
}

impl EnumPredicates<'_> {
    fn violations<'file>(&self, file: &'file SourceFile) -> Vec<Violation<'file>> {
        let mut scan = Predicates::new(self, file);
        scan.visit_file(file.syntax());
        scan.into_violations()
    }

    fn kind(&self, file: &SourceFile, module: &ModulePath, implemented: &Type) -> Kind {
        match implemented {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or(Kind::Other, |last| self.kind_named(file, module, path, &Name::from(&last.ident))),
            _ => Kind::Other,
        }
    }

    fn kind_named(&self, file: &SourceFile, module: &ModulePath, path: &TypePath, name: &Name) -> Kind {
        let crate_root = file.head().placement().crate_root();
        let local = Some(path)
            .filter(|candidate| candidate.qself.is_none())
            .filter(|unqualified| unqualified.path.segments.len() == 1)
            .and_then(|_| self.known.placed(crate_root, module, name));
        let enums = self.known.enumerated(crate_root, name);
        local.or(enums).unwrap_or(Kind::Other)
    }
}

pub struct Predicates<'scan, 'file> {
    known: &'scan EnumPredicates<'scan>,
    file: &'file SourceFile,
    module: Cow<'file, ModulePath>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> Predicates<'scan, 'file> {
    pub fn new(known: &'scan EnumPredicates<'scan>, file: &'file SourceFile) -> Predicates<'scan, 'file> {
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
