use super::ModulePath;
use super::relation::Relation;

pub struct Ancestor;

impl Relation for Ancestor {
    fn holds(this: &ModulePath, other: &ModulePath) -> bool {
        this.crate_root == other.crate_root
            && other.segments.get(..this.segments.len()) == Some(this.segments.as_slice())
    }
}
