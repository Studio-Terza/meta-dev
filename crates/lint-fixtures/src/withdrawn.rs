use std::convert::Infallible;
use std::str::FromStr;

use newtype::{Arbitrary, Newtype};

const WORDS: [&str; 6] = ["then", "then_some", "and_then", "map_or", "ok_or", "combinator"];
const FILTER: &str = "filter";
const SEPARATORS: [char; 9] = [' ', ',', '.', ';', ':', '(', ')', '`', '-'];
const PHRASES: [&str; 2] = ["pipeline of let bindings", "use combinators"];
const ELEMENT_FILTER: &str = "filter on a predicate that reads its element";
const FIX_START: &str = "; ";

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Spelling(String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Withdrawn(Vec<Spelling>);

impl FromStr for Withdrawn {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<Withdrawn, Infallible> {
        let at = line.rmatch_indices(FIX_START).next().map_or(0, |(at, _)| at + FIX_START.len());
        let text = line.get(at..).unwrap_or_default();
        let padded = format!(" {text} ");
        let hits: Vec<&str> = WORDS
            .iter()
            .copied()
            .chain([FILTER])
            .flat_map(|word| padded.match_indices(word))
            .filter(|(at, _)| {
                padded.get(..*at).and_then(|before| before.chars().next_back()).is_some_and(|c| SEPARATORS.contains(&c))
            })
            .filter(|(at, word)| {
                padded
                    .get(at + word.len()..)
                    .and_then(|after| after.chars().next())
                    .is_some_and(|c| SEPARATORS.contains(&c))
            })
            .map(|(_, word)| word)
            .collect();
        let filters = hits.iter().filter(|word| **word == FILTER).count();
        let surplus = std::iter::repeat_n(FILTER, filters.saturating_sub(padded.matches(ELEMENT_FILTER).count()));
        let words = hits.iter().copied().filter(|word| *word != FILTER).chain(surplus);
        let phrases = PHRASES.iter().copied().filter(|phrase| text.contains(phrase));
        Ok(Withdrawn(words.chain(phrases).map(|spelling| Spelling::new(String::from(spelling))).collect()))
    }
}
