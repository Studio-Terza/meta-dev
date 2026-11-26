use newtype::Listed;
use serde::Serialize;
use strum::{Display, IntoStaticStr, VariantArray};

#[derive(
    Display, IntoStaticStr, VariantArray, Listed, Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
#[strum(serialize_all = "kebab-case")]
pub enum HookEvent {
    Stop,
}
