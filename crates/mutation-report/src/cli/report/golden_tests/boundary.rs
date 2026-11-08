use std::str::FromStr;

use newtype::{Arbitrary, Newtype};

use crate::cli::report::test_name::TestName;

#[derive(Newtype, Arbitrary, Debug, PartialEq, Eq)]
pub struct GoldenTest(TestName);

#[derive(Debug, PartialEq, Eq)]
pub struct NotGolden;

#[derive(Debug)]
struct NameLetter;

impl NameLetter {
    const UNDERSCORE: char = '_';

    fn within_name(letter: char) -> Option<char> {
        Some(letter)
            .filter(|candidate| candidate.is_alphanumeric())
            .or_else(|| Some(letter).filter(|candidate| *candidate == Self::UNDERSCORE))
    }
}

impl GoldenTest {
    const NEXT_ATTRIBUTE: &'static str = "\n#[";
    const FUNCTION: &'static str = "fn";
    const MACRO: &'static str = "golden!";
    const TYPE_CALL: &'static str = "Golden::";
    const METHOD_CALL: &'static str = ".golden(";
    const OPEN: char = '(';

    fn uses_golden(text: &str) -> Option<&str> {
        let head = text.match_indices(Self::NEXT_ATTRIBUTE).next().and_then(|(at, _)| text.get(..at)).unwrap_or(text);
        [Self::TYPE_CALL, Self::METHOD_CALL]
            .iter()
            .find(|mark| head.contains(*mark))
            .copied()
            .or_else(|| head.match_indices(Self::MACRO).find_map(|(at, mark)| Self::opens(head.get(at + mark.len()..))))
            .map(|_| text)
    }

    fn opens(rest: Option<&str>) -> Option<&str> {
        rest.filter(|candidate| candidate.chars().skip_while(|letter| letter.is_whitespace()).take(1).eq([Self::OPEN]))
    }

    fn function_name(text: &str) -> Option<GoldenTest> {
        text.match_indices(Self::FUNCTION)
            .find_map(|(at, mark)| Self::after_keyword(text.get(at + mark.len()..)?))
            .map(|name| GoldenTest(TestName::new(name)))
    }

    fn after_keyword(rest: &str) -> Option<String> {
        let name: String =
            rest.chars().skip_while(|letter| letter.is_whitespace()).map_while(NameLetter::within_name).collect();
        rest.chars().next().filter(|letter| letter.is_whitespace()).and(name.chars().next()).map(|_| name)
    }
}

impl FromStr for GoldenTest {
    type Err = NotGolden;

    fn from_str(text: &str) -> Result<GoldenTest, NotGolden> {
        Self::uses_golden(text).and_then(Self::function_name).ok_or(NotGolden)
    }
}
