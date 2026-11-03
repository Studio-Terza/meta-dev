use super::ModulePath;
use super::relation::Relation;

pub struct Ancestor;

impl Relation for Ancestor {
    fn holds<'other, 'file>(
        this: &ModulePath<'_>,
        other: &'other ModulePath<'file>,
    ) -> Option<&'other ModulePath<'file>> {
        (this.crate_root == other.crate_root
            && other.segments.get(..this.segments.len()) == Some(this.segments.as_slice()))
        .then_some(other)
    }
}
