use lint_layout::Name;
use lint_paths::Module;
use lint_placement::CrateFile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Owner {
    root: Module,
    name: Name,
}

impl Owner {
    pub fn new(module: &Module, name: Name) -> Owner {
        Owner { root: Module::from(AsRef::<CrateFile>::as_ref(module)), name }
    }
}
