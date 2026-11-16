#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Statement(String);

impl From<String> for Statement {
    fn from(text: String) -> Statement {
        Statement(text)
    }
}

impl AsRef<str> for Statement {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
