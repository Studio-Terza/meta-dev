#[derive(
    strum::Display,
    strum::IntoStaticStr,
    strum::VariantArray,
    serde::Serialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[strum(serialize_all = "kebab-case")]
pub enum HookEvent {
    Stop,
    SubagentStop,
    PreToolUse,
    PostToolUse,
}
