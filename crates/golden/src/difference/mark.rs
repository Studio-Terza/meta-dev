use strum::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    #[strum(serialize = " ")]
    Context,
    #[strum(serialize = "-")]
    Recorded,
    #[strum(serialize = "+")]
    Actual,
}
