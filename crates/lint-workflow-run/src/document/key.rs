#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Run,
    Shell,
    Uses,
    With,
    Script,
}

impl Key {
    pub(crate) fn named(text: &str) -> Option<Key> {
        match text {
            "run" => Some(Key::Run),
            "shell" => Some(Key::Shell),
            "uses" => Some(Key::Uses),
            "with" => Some(Key::With),
            "script" => Some(Key::Script),
            _ => None,
        }
    }
}
