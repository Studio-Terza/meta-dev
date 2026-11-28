use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_test_log::TestLine;
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    lines: Vec<Spelled>,
}

#[test]
fn the_test_log_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let lines: Vec<_> = case.row().lines.iter().map(|text| (text, TestLine::read(text.as_str()))).collect();
        Ok(Debugged(lines).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
