use super::ModulePath;

pub trait Relation {
    fn holds<'other, 'file>(
        this: &ModulePath<'_>,
        other: &'other ModulePath<'file>,
    ) -> Option<&'other ModulePath<'file>>;
}
