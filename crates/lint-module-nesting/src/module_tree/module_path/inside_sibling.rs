use super::ModulePath;
use super::ancestor::Ancestor;

pub struct InsideSibling;

impl InsideSibling {
    pub fn holds<'module, 'file>(
        this: &ModulePath<'_>,
        module: &'module ModulePath<'file>,
    ) -> Option<&'module ModulePath<'file>> {
        let length = module.segments.len();
        module
            .parent()
            .and_then(|parent| Ancestor::holds(&parent, this).map(|_| module))
            .filter(|_| this.segments.len() >= length)
    }
}
