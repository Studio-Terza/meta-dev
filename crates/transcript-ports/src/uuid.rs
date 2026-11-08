use grammar::message::BareWord;
use grammar::output::Fields;
use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a message uuid, which is one bare word of letters, digits, '_' and '-', not starting with '-'")]
pub struct Unbare(String);

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(checked, strategy = "[A-Za-z0-9_][A-Za-z0-9_-]{0,36}")]
pub struct Uuid(String);

impl Checked for Uuid {
    type Error = Unbare;

    fn checked(text: &String) -> Result<&String, Unbare> {
        Fields::bare(text).map(|_bare| text).ok_or_else(|| Unbare(text.clone()))
    }
}

impl From<BareWord> for Uuid {
    fn from(word: BareWord) -> Uuid {
        Uuid(word.into_inner())
    }
}
