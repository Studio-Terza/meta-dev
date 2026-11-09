#![deny(clippy::wildcard_enum_match_arm)]

use grammar_spec_line::Attachment;
use laws::proptest::prelude::{Just, Strategy, any, prop_oneof};

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
