#![deny(clippy::wildcard_enum_match_arm)]

laws::roundtrip!(
    a_spec_line_without_trailing_space_is_parsed_and_shown_back_as_written,
    "repair: main|spec: ( ?[!-~]{1,8})( [!-~]{1,8}){0,3}",
    |text: &String| text.parse::<grammar_spec_line::Line>().ok(),
    |line: &Option<grammar_spec_line::Line>| line.as_ref().map(ToString::to_string),
    Some
);
