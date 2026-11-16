use syntax::{Pat, PatTree, Path};

use lint_layout::Name;

#[derive(Debug)]
pub struct VariantPaths;

impl VariantPaths {
    pub fn in_pat(pat: &Pat) -> Vec<Name> {
        let reached = PatTree::reached(pat, &|_, _| true);
        reached.into_iter().filter_map(|found| Self::path(found.pat())).filter_map(Self::parent).collect()
    }

    fn path(pat: &Pat) -> Option<&Path> {
        match pat {
            Pat::Path(path) => Some(&path.path),
            Pat::TupleStruct(variant) => Some(&variant.path),
            Pat::Struct(variant) => Some(&variant.path),
            _ => None,
        }
    }

    fn parent(path: &Path) -> Option<Name> {
        path.segments.iter().nth_back(1).map(|parent| Name::from(&parent.ident))
    }
}
