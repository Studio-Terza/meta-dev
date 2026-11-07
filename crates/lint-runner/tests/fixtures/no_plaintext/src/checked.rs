use serde::Deserialize;

#[derive(newtype::Newtype, newtype::Arbitrary, newtype::Deserialize)]
#[newtype(checked)]
pub struct Title(String);

pub struct Untitled;

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
