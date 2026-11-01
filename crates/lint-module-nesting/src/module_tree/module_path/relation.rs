use super::ModulePath;

pub trait Relation {
    fn holds(this: &ModulePath, other: &ModulePath) -> bool;
}
