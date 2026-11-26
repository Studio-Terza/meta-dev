use newtype::Spelled;
use strum::{Display, EnumString};

#[derive(Spelled, Display, EnumString, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case", parse_err_ty = Unvoiced, parse_err_fn = Unvoiced::from)]
pub enum Authorship {
    Owner,
    Lead,
}

#[derive(Debug, thiserror::Error)]
#[error("{0:?} names neither the owner nor the lead")]
pub struct Unvoiced(String);

impl From<&str> for Unvoiced {
    fn from(text: &str) -> Unvoiced {
        Unvoiced(text.to_owned())
    }
}
