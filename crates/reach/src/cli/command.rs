use strum::EnumString;

#[derive(EnumString, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Command {
    Affected,
    Widened,
    History,
    Sources,
    Beyond,
}
