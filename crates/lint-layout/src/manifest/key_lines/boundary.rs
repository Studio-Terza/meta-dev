#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TableKey {
    PortOwns,
    LexerFormat,
}

impl TableKey {
    pub(super) const ALL: [TableKey; 2] = [TableKey::PortOwns, TableKey::LexerFormat];

    pub(super) fn table(self) -> [&'static str; 3] {
        match self {
            TableKey::PortOwns => ["package", "metadata", "port"],
            TableKey::LexerFormat => ["package", "metadata", "lexer"],
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            TableKey::PortOwns => "owns",
            TableKey::LexerFormat => "format",
        }
    }
}
