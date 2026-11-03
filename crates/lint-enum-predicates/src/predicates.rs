mod declaration;
mod declarations;
mod finding;
mod kind;

use std::borrow::Cow;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, Macro, ReturnType, Type, TypePath};

use lint_core::{Headline, Lint, Macros, Report, Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name};
use lint_placement::ModulePath;

use declaration::Declaration;
use declarations::Declarations;
use finding::Finding;
use kind::Kind;

#[derive(Debug, Default)]
pub struct EnumPredicates<'file> {
    declarations: Vec<Declaration<'file>>,
}

impl Lint for EnumPredicates<'_> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let predicates = EnumPredicates { declarations: files.iter().flat_map(Declarations::in_file).collect() };
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
        let crate_root = file.placement().crate_root();
        let local = (path.qself.is_none() && path.path.segments.len() == 1)
            .then(|| {
                self.declarations
                    .iter()
                    .filter_map(|declaration| declaration.in_module(module))
                    .find_map(|declaration| declaration.kinds(crate_root, name).next())
            })
            .flatten();
        let candidates: Vec<Kind> =
            self.declarations.iter().flat_map(|declaration| declaration.kinds(crate_root, name)).collect();
        match (local, candidates.as_slice()) {
            (Some(kind), _) => kind,
            (None, all @ [_, ..]) if all.iter().all(|kind| *kind == Kind::Enum) => Kind::Enum,
            (None, _) => Kind::Other,
        }
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
        Predicates { known, file, module: Cow::Borrowed(file.placement().module()), violations: Vec::new() }
    }

    pub fn into_violations(self) -> Vec<Violation<'file>> {
        self.violations
    }

    fn predicates(&self, block: &ItemImpl) -> Vec<Violation<'file>> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(&function.sig),
                _ => None,
            })
            .filter(|signature| match &signature.output {
                ReturnType::Type(_, returned) => {
                    matches!(returned.as_ref(), Type::Path(path) if path.path.is_ident("bool"))
                }
                ReturnType::Default => false,
            })
            .map(|signature| {
                Violation::new(
                    Site::new(self.file.path(), LineNumber::from(signature.span())),
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
        let found = match (&block.trait_, self.known.kind(self.file, &self.module, &block.self_ty)) {
            (None, Kind::Enum) => self.predicates(block),
            _ => Vec::new(),
        };
        self.violations.extend(found);
        syn::visit::visit_item_impl(self, block);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}
