use lint_layout::Name;
use lint_paths::Module;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeKey<'guards> {
    module: &'guards Module<'guards>,
    name: &'guards Name,
}

impl<'guards> TypeKey<'guards> {
    pub const fn new(module: &'guards Module<'guards>, name: &'guards Name) -> Self {
        TypeKey { module, name }
    }
}
