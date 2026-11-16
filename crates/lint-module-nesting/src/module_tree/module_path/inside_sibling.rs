use lint_placement::Level;

use super::ModulePath;

pub struct InsideSibling;

impl InsideSibling {
    pub fn holds<'module, 'file>(
        this: &ModulePath<'_>,
        module: &'module ModulePath<'file>,
    ) -> Option<&'module ModulePath<'file>> {
        let length = module.segments().len();
        module
            .nest
            .parent()
            .filter(|parent| parent.level() != Level::Workspace)
            .and_then(|parent| parent.holding(this.nest).map(|_| module))
            .filter(|_| this.segments().len() >= length)
    }
}
