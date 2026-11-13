use lint_core::Module;
use lint_layout::Name;
use lint_placement::CrateFile;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Owner<'file> {
    root: Module<'file>,
    name: Name,
}

impl<'file> Owner<'file> {
    pub fn new(module: &Module<'file>, name: Name) -> Owner<'file> {
        Owner { root: Module::from(<&CrateFile>::from(module)), name }
    }
}
