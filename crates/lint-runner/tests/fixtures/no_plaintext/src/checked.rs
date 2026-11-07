use serde::Deserialize;

#[derive(newtype::Newtype, newtype::Arbitrary, newtype::CheckedDeserialize, Debug)]
#[newtype(checked)]
pub struct Title(String);

#[derive(newtype::Newtype, newtype::Display, Debug)]
pub struct Untitled(&'static str);

impl newtype::Checked for Title {
    type Error = Untitled;

    fn checked(text: &String) -> Result<&String, Untitled> {
        Ok(text)
    }
}

#[derive(Deserialize)]
pub struct Entry {
    title: Title,
}
