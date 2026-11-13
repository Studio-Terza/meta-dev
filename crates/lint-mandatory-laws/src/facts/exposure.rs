#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exposure {
    Private,
    Public,
}

impl From<code_model::Exposure> for Exposure {
    fn from(exposure: code_model::Exposure) -> Exposure {
        match exposure {
            code_model::Exposure::Public => Exposure::Public,
            code_model::Exposure::Hidden => Exposure::Private,
        }
    }
}
