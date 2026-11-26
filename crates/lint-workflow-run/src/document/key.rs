use newtype::Wrapping;
use strum::{IntoStaticStr, VariantArray};

use crate::document::text::Text;

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Key {
    Run,
    Shell,
    Uses,
    With,
    Script,
    Env,
}

impl Key {
    pub(crate) fn named(text: Text<'_>) -> Option<Key> {
        Self::VARIANTS.iter().copied().find(|key| <&str>::from(*key) == text.into_inner())
    }
}
