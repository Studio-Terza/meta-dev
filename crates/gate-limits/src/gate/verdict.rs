use strum::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    #[strum(serialize = "ok")]
    Within,
    #[strum(serialize = "WARN")]
    Warn,
    #[strum(serialize = "BLOCK")]
    Block,
}
