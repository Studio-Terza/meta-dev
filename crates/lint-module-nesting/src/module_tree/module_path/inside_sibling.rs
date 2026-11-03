use super::ModulePath;
use super::ancestor::Ancestor;
use super::relation::Relation;

pub struct InsideSibling;

impl Relation for InsideSibling {
    fn holds<'module, 'file>(
        this: &ModulePath<'_>,
        module: &'module ModulePath<'file>,
    ) -> Option<&'module ModulePath<'file>> {
        let length = module.segments.len();
        module
            .parent()
            .and_then(|parent| parent.relates::<Ancestor>(this).map(|_| module))
            .filter(|_| this.segments.len() >= length)
    }
}
