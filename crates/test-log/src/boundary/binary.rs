use std::str::FromStr;

use grammar::banner::Banner;

use super::unmatched::Unmatched;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binary(String);

impl Binary {
    pub(crate) fn label(&self) -> &str {
        &self.0
    }
}

impl FromStr for Binary {
    type Err = Unmatched;

    fn from_str(line: &str) -> Result<Binary, Unmatched> {
        Banner::try_from(line).map(|banner| Binary(banner.to_string())).map_err(|_unbannered| Unmatched)
    }
}
