#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exposure {
    Private,
    Public,
}

impl From<graph::Exposure> for Exposure {
    fn from(exposure: graph::Exposure) -> Exposure {
        match exposure {
            graph::Exposure::Public => Exposure::Public,
            graph::Exposure::Hidden => Exposure::Private,
        }
    }
}
