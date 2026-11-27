#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub enum Model {
    Haiku,
    Sonnet,
}

impl Model {
    #[must_use]
    pub(crate) fn spelled(self) -> [&'static str; 2] {
        match self {
            Model::Haiku => ["--model", "haiku"],
            Model::Sonnet => ["--model", "sonnet"],
        }
    }
}
