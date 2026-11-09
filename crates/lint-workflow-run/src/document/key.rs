use newtype::Wrapping;

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
    const NAMES: [(&'static str, Key); 6] = [
        ("run", Key::Run),
        ("shell", Key::Shell),
        ("uses", Key::Uses),
        ("with", Key::With),
        ("script", Key::Script),
        ("env", Key::Env),
    ];

    pub(crate) fn named(text: Text<'_>) -> Option<Key> {
        Self::NAMES.into_iter().find(|(name, _)| *name == text.into_inner()).map(|(_, key)| key)
    }
}
