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
    pub(crate) fn named(text: Text<'_>) -> Option<Key> {
        match text.into_inner() {
            "run" => Some(Key::Run),
            "shell" => Some(Key::Shell),
            "uses" => Some(Key::Uses),
            "with" => Some(Key::With),
            "script" => Some(Key::Script),
            "env" => Some(Key::Env),
            _ => None,
        }
    }
}
