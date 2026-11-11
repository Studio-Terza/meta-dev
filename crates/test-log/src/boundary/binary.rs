use std::str::FromStr;

use grammar_banner::Banner;

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
        line.parse::<Banner>().map(|banner| Binary(banner.into_inner())).map_err(|_unbannered| Unmatched)
    }
}
