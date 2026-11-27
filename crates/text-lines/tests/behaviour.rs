use proptest::prelude::{any, prop_assert_eq, proptest};
use text_lines::{Lines, LossyText, OneLine};

const NEWLINE: &str = "\n";

proptest! {
    #[test]
    fn lines_joined_by_their_breaks_give_back_the_text(text in "[a-zé \t\r\n]{0,24}") {
        let joined = text.parse::<Lines>().into_iter().flatten().collect::<Vec<String>>().join(NEWLINE);
        let rebuilt = [joined.clone(), [joined, NEWLINE.to_owned()].concat()];
        prop_assert_eq!(rebuilt.iter().filter(|candidate| **candidate == text).count(), 1);
    }

    #[test]
    fn utf8_bytes_read_as_the_text_they_spell(text in any::<String>()) {
        prop_assert_eq!(LossyText::from(text.as_bytes()).to_string(), text);
    }

    #[test]
    fn text_is_one_line_exactly_when_it_holds_no_control_character(text in any::<String>()) {
        let line = OneLine::try_new(text.clone()).map(|read| read.to_string());
        let expected = text.chars().all(|letter| !letter.is_control()).then(|| text.clone());
        prop_assert_eq!(line.into_iter().next(), expected);
    }

    #[test]
    fn a_line_read_from_json_is_checked_the_same_way(text in any::<String>()) {
        let json = serde_json::to_string(&text).into_iter().next().unwrap_or_default();
        let read = serde_json::from_str::<OneLine>(&json).map(|line| line.to_string());
        let direct = OneLine::try_new(text).map(|line| line.to_string());
        prop_assert_eq!(read.into_iter().next(), direct.into_iter().next());
    }
}
