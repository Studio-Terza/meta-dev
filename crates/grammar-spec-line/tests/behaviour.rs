#![deny(clippy::wildcard_enum_match_arm)]

use grammar_spec_line::Attachment;
use laws::proptest::prelude::{Just, Strategy, any, prop_assert_eq, prop_oneof};

laws::roundtrip!(
    a_spec_line_without_trailing_space_is_parsed_and_shown_back_as_written,
    "repair: main|spec: ( ?[!-~]{1,8})( [!-~]{1,8}){0,3}",
    |text: &String| text.parse::<grammar_spec_line::Line>().ok(),
    |line: &Option<grammar_spec_line::Line>| line.as_ref().map(ToString::to_string),
    Some
);

laws::roundtrip!(
    a_spec_line_reads_back_from_its_rendering,
    prop_oneof![
        Just("repair: main".to_owned()),
        any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
        "[!-~]([ -~]{0,20}[!-~])?".prop_map(|value| format!("spec: {value}")),
    ],
    |text: &String| text.parse::<grammar_spec_line::Line>().ok(),
    |line: &Option<grammar_spec_line::Line>| line.as_ref().map(ToString::to_string),
    Some
);

laws::law!(
    any_text_the_spec_line_lexer_accepts_reads_back_from_its_rendering,
    prop_oneof![
        any::<String>(),
        "(spec: |repair: main)?[a-z0-9#, \t\r\n-]{0,12}",
        any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
    ],
    text,
    {
        let line = text.parse::<grammar_spec_line::Line>().ok();
        let again = line.as_ref().and_then(|read| read.to_string().parse::<grammar_spec_line::Line>().ok());
        prop_assert_eq!(again, line);
    }
);
