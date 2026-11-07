use syn::Visibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exposure {
    Private,
    Public,
}

impl Exposure {
    pub(crate) const fn declared_by(visibility: &Visibility) -> Self {
        match visibility {
            Visibility::Public(_) => Self::Public,
            _ => Self::Private,
        }
    }
}
