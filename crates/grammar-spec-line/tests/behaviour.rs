#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_spec_line::{Attachment, Line};
use proptest::prelude::{Just, Strategy, any, prop_assert_eq, prop_oneof, proptest};
use proptest::test_runner::{Config, RngSeed};

struct Spec;

impl Spec {
    fn read(text: &str) -> Option<Line> {
        text.parse::<Line>().ignoring(Unlexed::NotThisGrammar)
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_spec_line_without_trailing_space_is_parsed_and_shown_back_as_written(
        text in "repair: main|spec: ( ?[!-~]{1,8})( [!-~]{1,8}){0,3}"
    ) {
        prop_assert_eq!(Spec::read(&text).as_ref().map(ToString::to_string), Some(text));
    }

    #[test]
    fn a_spec_line_reads_back_from_its_rendering(
        text in prop_oneof![
            Just("repair: main".to_owned()),
            any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
            "[!-~]([ -~]{0,20}[!-~])?".prop_map(|value| format!("spec: {value}")),
        ]
    ) {
        prop_assert_eq!(Spec::read(&text).as_ref().map(ToString::to_string), Some(text));
    }

    #[test]
    fn any_text_the_spec_line_lexer_accepts_reads_back_from_its_rendering(
        text in prop_oneof![
            any::<String>(),
            "(spec: |repair: main)?[a-z0-9#, \t\r\n-]{0,12}",
            any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
        ]
    ) {
        let line = Spec::read(&text);
        let again = line.as_ref().and_then(|read| Spec::read(&read.to_string()));
        prop_assert_eq!(again, line);
    }
}
