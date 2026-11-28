#![deny(clippy::wildcard_enum_match_arm)]

use fixture::{Failed, Fixture, Properties, Rows, Spelled};
use grammar::{Ignoring, Unlexed};
use grammar_spec_line::Line;
use proptest::prelude::{any, prop_assert_eq};
use proptest::string::string_regex;
use proptest::test_runner::{TestCaseError, TestRunner};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Exact {
    shape: Spelled,
}

#[derive(Fixture, Deserialize, Debug)]
struct Steady {
    shape: Spelled,
}

#[test]
fn every_exact_shape_reads_as_a_line_and_shows_back_as_written() -> Result<(), Failed> {
    Rows::<Exact>::read()?.iter().try_for_each(|exact| {
        TestRunner::new(Properties::settled()).run(&string_regex(exact.shape.as_str())?, |text| {
            let line = text.parse::<Line>().ignoring(Unlexed::NotThisGrammar);
            prop_assert_eq!(line.as_ref().map(ToString::to_string), Some(text));
            Ok(())
        })?;
        Ok(())
    })
}

#[test]
fn any_text_the_lexer_accepts_reads_back_from_its_rendering() -> Result<(), Failed> {
    let steady = |text: String| {
        let line = text.parse::<Line>().ignoring(Unlexed::NotThisGrammar);
        let again = line.as_ref().and_then(|read| read.to_string().parse::<Line>().ignoring(Unlexed::NotThisGrammar));
        prop_assert_eq!(again, line);
        Ok::<(), TestCaseError>(())
    };
    TestRunner::new(Properties::settled()).run(&any::<String>(), steady)?;
    Rows::<Steady>::read()?.iter().try_for_each(|shaped| {
        TestRunner::new(Properties::settled()).run(&string_regex(shaped.shape.as_str())?, steady)?;
        Ok(())
    })
}
