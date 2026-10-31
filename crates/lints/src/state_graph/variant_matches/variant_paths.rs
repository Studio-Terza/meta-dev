use syn::visit::Visit;
use syn::{ExprPath, Pat, PatStruct, PatTupleStruct, Path};

use lint_core::Name;

#[derive(Default)]
pub struct VariantPaths(Vec<Name>);

impl VariantPaths {
    pub fn in_pat(pat: &Pat) -> Vec<Name> {
        let mut paths = VariantPaths::default();
        paths.visit_pat(pat);
        paths.0
    }

    fn record(&mut self, path: &Path) {
        self.0.extend(path.segments.iter().nth_back(1).map(|parent| Name::from(&parent.ident)));
    }
}

impl<'ast> Visit<'ast> for VariantPaths {
    fn visit_expr_path(&mut self, pat: &'ast ExprPath) {
        self.record(&pat.path);
    }

    fn visit_pat_tuple_struct(&mut self, pat: &'ast PatTupleStruct) {
        self.record(&pat.path);
        syn::visit::visit_pat_tuple_struct(self, pat);
    }

    fn visit_pat_struct(&mut self, pat: &'ast PatStruct) {
        self.record(&pat.path);
        syn::visit::visit_pat_struct(self, pat);
    }
}
