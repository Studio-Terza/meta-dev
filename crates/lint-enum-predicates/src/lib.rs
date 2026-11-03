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
pub struct EnumPredicates {
    declarations: Vec<Declaration>,
}

impl Lint for EnumPredicates {
    fn report(files: &[SourceFile]) -> Report {
        let predicates = EnumPredicates { declarations: files.iter().flat_map(Declarations::in_file).collect() };
        Report::new(
            Headline::new(
                "an enum method returning bool throws away the variant; return an enum or match where the answer is used",
            ),
            files.iter().flat_map(|file| predicates.violations(file)).collect(),
        )
    }
}

impl EnumPredicates {
    fn violations(&self, file: &SourceFile) -> Vec<Violation> {
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
        let candidates: Vec<&Declaration> = self
            .declarations
            .iter()
            .filter_map(|declaration| declaration.named(file.placement().crate_root(), name))
            .collect();
        let local = (path.qself.is_none() && path.path.segments.len() == 1)
            .then(|| candidates.iter().find_map(|declaration| declaration.in_module(module)))
            .flatten();
        match (local, candidates.as_slice()) {
            (Some(declaration), _) => declaration.kind(),
            (None, []) => Kind::Other,
            (None, all) if all.iter().all(|declaration| declaration.kind() == Kind::Enum) => Kind::Enum,
            (None, _) => Kind::Other,
        }
    }
}
