use syn::{ItemImpl, Type};

use lint_layout::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Owner {
    Named(Name),
    #[default]
    Free,
}

impl Owner {
    pub fn of_impl(item: &ItemImpl) -> Owner {
        match &*item.self_ty {
            Type::Path(path) => {
                path.path.segments.last().map_or(Owner::Free, |segment| Owner::Named(Name::from(&segment.ident)))
            }
            _ => Owner::Free,
        }
    }
}
