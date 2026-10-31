mod declaration;
mod declarations;
mod finding;
mod kind;
mod predicates;

use declaration::Declaration;
use declarations::Declarations;
use finding::Finding;
use kind::Kind;
use predicates::Predicates;
use syn::Type;
use syn::visit::Visit;

use lint_core::{ModulePath, Name, Report, SourceFile, Violation};

#[derive(Debug, Default)]
pub struct EnumPredicates {
    declarations: Vec<Declaration>,
}

impl EnumPredicates {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let predicates = EnumPredicates { declarations: files.iter().flat_map(Declarations::of).collect() };
        Report::new(
            "an enum method returning bool throws away the variant; return an enum or match where the answer is used",
            files.iter().flat_map(|file| predicates.violations(file)).collect(),
        )
    }

    fn violations(&self, file: &SourceFile) -> Vec<Violation> {
        let mut scan = Predicates::new(self, file);
        scan.visit_file(file.syntax());
        scan.into_violations()
    }

    fn is_enum(&self, file: &SourceFile, module: &ModulePath, implemented: &Type) -> bool {
        let Type::Path(path) = implemented else {
            return false;
        };
        let Some(last) = path.path.segments.last() else {
            return false;
        };
        let name = Name::from(&last.ident);
        let candidates: Vec<&Declaration> =
            self.declarations.iter().filter(|declaration| declaration.is_named(file.crate_root(), &name)).collect();
        let local = (path.qself.is_none() && path.path.segments.len() == 1)
            .then(|| candidates.iter().find(|declaration| declaration.is_in(module)))
            .flatten();
        match local {
            Some(declaration) => declaration.kind() == Kind::Enum,
            None => !candidates.is_empty() && candidates.iter().all(|declaration| declaration.kind() == Kind::Enum),
        }
    }
}
