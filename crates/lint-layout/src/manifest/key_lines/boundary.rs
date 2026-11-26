use strum::{IntoStaticStr, VariantArray};

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TableKey {
    #[strum(serialize = "owns")]
    PortOwns,
    #[strum(serialize = "format")]
    LexerFormat,
}

impl TableKey {
    pub(super) fn table(self) -> [&'static str; 3] {
        match self {
            TableKey::PortOwns => ["package", "metadata", "port"],
            TableKey::LexerFormat => ["package", "metadata", "lexer"],
        }
    }
}
