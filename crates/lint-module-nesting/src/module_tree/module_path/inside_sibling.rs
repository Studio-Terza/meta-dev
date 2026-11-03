use super::ModulePath;
use super::ancestor::Ancestor;
use super::relation::Relation;

pub struct InsideSibling;

impl Relation for InsideSibling {
    fn holds<'module>(this: &ModulePath, module: &'module ModulePath) -> Option<&'module ModulePath> {
        let length = module.segments.len();
        module
            .parent()
            .and_then(|parent| parent.relates::<Ancestor>(this).map(|_| module))
            .filter(|_| this.segments.len() >= length)
    }
}
