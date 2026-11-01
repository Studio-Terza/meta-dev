use std::convert::Infallible;
use std::str::FromStr;

pub(super) struct Part {
    name: String,
    lowered: String,
}

impl Part {
    const WHOLE: [&'static str; 2] = ["test", "tests"];

    const ENDINGS: [&'static str; 2] = ["Test", "Tests"];

    const EXTENSION: char = '.';

    pub(super) fn names_tests(&self) -> bool {
        Self::WHOLE.contains(&self.lowered.as_str())
            || Self::ENDINGS.iter().any(|ending| {
                self.name.len().checked_sub(ending.len()).and_then(|from| self.name.get(from..)) == Some(*ending)
            })
    }
}

impl FromStr for Part {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Part, Infallible> {
        let name: String = text.chars().take_while(|letter| *letter != Self::EXTENSION).collect();
        let lowered = name.chars().flat_map(char::to_lowercase).collect();
        Ok(Part { name, lowered })
    }
}
