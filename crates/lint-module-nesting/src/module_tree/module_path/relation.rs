use super::ModulePath;

pub trait Relation {
    fn holds<'other>(this: &ModulePath, other: &'other ModulePath) -> Option<&'other ModulePath>;
}
