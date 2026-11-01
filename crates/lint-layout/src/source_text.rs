use newtype::Newtype;

#[derive(Newtype, Clone, Debug, PartialEq, Eq)]
pub struct SourceText(String);

impl AsRef<str> for SourceText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
