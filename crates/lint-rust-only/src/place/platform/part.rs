use std::convert::Infallible;
use std::str::FromStr;

pub(super) struct Part(String);

#[derive(Clone, Copy)]
struct Marker(&'static str);

impl Part {
    const WHOLE: [Marker; 2] = [Marker("test"), Marker("tests")];

    const ENDINGS: [Marker; 2] = [Marker("Test"), Marker("Tests")];

    const EXTENSION: char = '.';

    pub(super) fn naming_tests(self) -> Option<Part> {
        let lowered: String = self.0.chars().flat_map(char::to_lowercase).collect();
        Self::WHOLE
            .into_iter()
            .find(|whole| lowered == whole.0)
            .or_else(|| Self::ENDINGS.into_iter().find_map(|ending| self.ending(ending)))
            .map(|_| self)
    }

    fn ending(&self, ending: Marker) -> Option<Marker> {
        self.0
            .len()
            .checked_sub(ending.0.len())
            .and_then(|from| self.0.get(from..))
            .filter(|tail| *tail == ending.0)
            .map(|_| ending)
    }
}

impl FromStr for Part {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Part, Infallible> {
        Ok(Part(text.chars().take_while(|letter| *letter != Self::EXTENSION).collect()))
    }
}
