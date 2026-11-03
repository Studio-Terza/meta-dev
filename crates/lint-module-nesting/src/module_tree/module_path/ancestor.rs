use super::ModulePath;

pub struct Ancestor;

impl Ancestor {
    pub fn holds<'other, 'file>(
        this: &ModulePath<'_>,
        other: &'other ModulePath<'file>,
    ) -> Option<&'other ModulePath<'file>> {
        (this.crate_root == other.crate_root && other.segments.get(..this.segments.len()) == Some(this.segments))
            .then_some(other)
    }
}
