use newtype::Newtype;

use crate::document::text::Text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Run,
    Shell,
    Uses,
    With,
    Script,
    Env,
}

impl Key {
    const NAMES: [(&'static str, Self); 6] = [
        ("run", Self::Run),
        ("shell", Self::Shell),
        ("uses", Self::Uses),
        ("with", Self::With),
        ("script", Self::Script),
        ("env", Self::Env),
    ];

    pub(crate) fn named(text: Text<'_>) -> Option<Self> {
        Self::NAMES.into_iter().find(|(name, _)| *name == text.into_inner()).map(|(_, key)| key)
    }
}
