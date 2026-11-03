use super::ModulePath;
use super::relation::Relation;

pub struct Ancestor;

impl Relation for Ancestor {
    fn holds<'other, 'file>(
        this: &ModulePath<'_>,
        other: &'other ModulePath<'file>,
    ) -> Option<&'other ModulePath<'file>> {
        Some(other)
            .filter(|other| this.crate_root == other.crate_root)
            .filter(|other| other.segments.get(..this.segments.len()) == Some(this.segments))
    }
}
