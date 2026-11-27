use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a message uuid, which is one bare word of letters, digits, '_' and '-', not starting with '-'")]
pub struct Unbare(String);

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(checked, strategy = "[A-Za-z0-9_][A-Za-z0-9_-]{0,36}")]
pub struct Uuid(String);

impl Uuid {
    const JOINER: char = '_';
    const DASH: char = '-';
}

impl Checked for Uuid {
    type Error = Unbare;

    fn checked(text: &String) -> Result<&String, Unbare> {
        let first = text.chars().next().filter(|letter| letter.is_ascii_alphanumeric() || *letter == Uuid::JOINER);
        let rest = text
            .chars()
            .skip(1)
            .all(|letter| letter.is_ascii_alphanumeric() || letter == Uuid::JOINER || letter == Uuid::DASH);
        first.filter(|_| rest).map(|_| text).ok_or_else(|| Unbare(text.clone()))
    }
}
