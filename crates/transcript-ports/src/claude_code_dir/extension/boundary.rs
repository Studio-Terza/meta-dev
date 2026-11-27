use strum::EnumString;

#[derive(Debug, thiserror::Error)]
#[error("not a transcript file extension")]
pub(crate) struct Foreign;

impl From<&str> for Foreign {
    fn from(_name: &str) -> Foreign {
        Foreign
    }
}

#[derive(EnumString, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case", parse_err_ty = Foreign, parse_err_fn = Foreign::from)]
pub(crate) enum Extension {
    Jsonl,
}
