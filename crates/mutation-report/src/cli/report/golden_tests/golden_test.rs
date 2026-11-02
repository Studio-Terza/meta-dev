use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct GoldenTest(String);

#[derive(Debug, PartialEq, Eq)]
pub struct NotGolden;

impl GoldenTest {
    const NEXT_ATTRIBUTE: &'static str = "\n#[";
    const FUNCTION: &'static str = "fn";
    const MACRO: &'static str = "golden!";
    const TYPE_CALL: &'static str = "Golden::";
    const METHOD_CALL: &'static str = ".golden(";
    const OPEN: char = '(';
    const UNDERSCORE: char = '_';

    pub(crate) fn into_name(self) -> String {
        self.0
    }

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
        rest.filter(|rest| rest.chars().find(|letter| !letter.is_whitespace()) == Some(Self::OPEN))
    }

    fn function_name(text: &str) -> Option<GoldenTest> {
        text.match_indices(Self::FUNCTION)
            .find_map(|(at, mark)| Self::after_keyword(text.get(at + mark.len()..)?))
            .map(GoldenTest)
    }

    fn after_keyword(rest: &str) -> Option<String> {
        let name: String = rest
            .chars()
            .skip_while(|letter| letter.is_whitespace())
            .take_while(|letter| letter.is_alphanumeric() || *letter == Self::UNDERSCORE)
            .collect();
        (rest.chars().next().is_some_and(char::is_whitespace) && !name.is_empty()).then_some(name)
    }
}

impl FromStr for GoldenTest {
    type Err = NotGolden;

    fn from_str(text: &str) -> Result<GoldenTest, NotGolden> {
        Self::uses_golden(text).and_then(Self::function_name).ok_or(NotGolden)
    }
}
