use syn::{ExprPath, Pat, PatStruct, PatTupleStruct, Path};

use lint_layout::Name;
use lint_sweep::{Descend, Kind, Sweep, Watch};

#[derive(Debug, Default)]
pub struct VariantPaths(Vec<Name>);

impl VariantPaths {
    pub fn in_pat(pat: &Pat) -> Vec<Name> {
        let mut paths = VariantPaths::default();
        Sweep::through_pat(pat, &mut paths);
        paths.0
    }

    fn record(&mut self, path: &Path, descend: Descend) -> Descend {
        self.0.extend(path.segments.iter().nth_back(1).map(|parent| Name::from(&parent.ident)));
        descend
    }
}

impl<'ast> Watch<'ast> for VariantPaths {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ExprPath, Kind::PatTupleStruct, Kind::PatStruct]
    }

    fn on_expr_path(&mut self, pat: &'ast ExprPath) -> Descend {
        self.record(&pat.path, Descend::Over)
    }

    fn on_pat_tuple_struct(&mut self, pat: &'ast PatTupleStruct) -> Descend {
        self.record(&pat.path, Descend::Into)
    }

    fn on_pat_struct(&mut self, pat: &'ast PatStruct) -> Descend {
        self.record(&pat.path, Descend::Into)
    }
}
