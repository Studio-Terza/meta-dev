use super::ModulePath;
use super::relation::Relation;

pub struct Same;

impl Relation for Same {
    fn holds<'other>(this: &ModulePath, other: &'other ModulePath) -> Option<&'other ModulePath> {
        (this.crate_root == other.crate_root && this.segments == other.segments).then_some(other)
    }
}
