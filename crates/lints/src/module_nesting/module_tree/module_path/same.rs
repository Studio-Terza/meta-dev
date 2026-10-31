use super::ModulePath;
use super::relation::Relation;

pub struct Same;

impl Relation for Same {
    fn holds(this: &ModulePath, other: &ModulePath) -> bool {
        this.crate_root == other.crate_root && this.segments == other.segments
    }
}
