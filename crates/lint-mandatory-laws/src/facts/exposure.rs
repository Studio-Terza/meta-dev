use syn::Visibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exposure {
    Private,
    Public,
}

impl Exposure {
    pub(crate) fn declared_by(visibility: &Visibility) -> Exposure {
        match visibility {
            Visibility::Public(_) => Exposure::Public,
            _ => Exposure::Private,
        }
    }
}
