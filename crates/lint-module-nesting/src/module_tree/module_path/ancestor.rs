use super::ModulePath;

pub struct Ancestor;

impl Ancestor {
    pub fn holds<'other, 'file>(
        this: &ModulePath<'_>,
        other: &'other ModulePath<'file>,
    ) -> Option<&'other ModulePath<'file>> {
        Some(other)
            .filter(|candidate| this.crate_root == candidate.crate_root)
            .filter(|in_crate| in_crate.segments.get(..this.segments.len()) == Some(this.segments))
    }
}
