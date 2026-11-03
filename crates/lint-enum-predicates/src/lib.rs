mod declaration;
mod declarations;
mod finding;
mod kind;
mod predicates;

use declaration::Declaration;
use declarations::Declarations;
use finding::Finding;
use kind::Kind;
use lint_core::Lint;
use predicates::Predicates;
use syn::visit::Visit;
use syn::{Type, TypePath};

use lint_core::{Headline, Report, SourceFile, Violation};
use lint_layout::Name;
use lint_placement::ModulePath;

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
        let local = Some(path)
            .filter(|path| path.qself.is_none())
            .filter(|path| path.path.segments.len() == 1)
            .and_then(|_| {
                self.declarations
                    .iter()
                    .filter_map(|declaration| declaration.in_module(module))
                    .find_map(|declaration| declaration.kinds(crate_root, name).next())
            });
        let candidates: Vec<Kind> =
            self.declarations.iter().flat_map(|declaration| declaration.kinds(crate_root, name)).collect();
        let enums =
            candidates.first().filter(|_| candidates.iter().all(|kind| *kind == Kind::Enum)).map(|_| Kind::Enum);
        local.or(enums).unwrap_or(Kind::Other)
    }
}
