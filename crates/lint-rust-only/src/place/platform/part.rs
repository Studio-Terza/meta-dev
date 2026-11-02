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

    pub(super) fn naming_tests(self) -> Option<Part> {
        Self::WHOLE
            .iter()
            .copied()
            .find(|whole| self.lowered == *whole)
            .or_else(|| Self::ENDINGS.iter().find_map(|ending| self.ending(ending)))
            .map(|_| self)
    }

    fn ending(&self, ending: &'static str) -> Option<&'static str> {
        self.name
            .len()
            .checked_sub(ending.len())
            .and_then(|from| self.name.get(from..))
            .filter(|tail| *tail == ending)
            .map(|_| ending)
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
