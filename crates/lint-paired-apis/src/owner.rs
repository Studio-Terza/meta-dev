use lint_layout::Name;
use lint_paths::Module;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Owner {
    root: Module,
    name: Name,
}

impl Owner {
    pub fn new(module: &Module, name: Name) -> Owner {
        Owner { root: module.root(), name }
    }
}
