use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_letters::Letters;
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    slugs: Vec<Spelled>,
}

#[test]
fn the_letters_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let slugs: Vec<_> = case.row().slugs.iter().map(|text| (text, Letters::slugged(text.as_str()))).collect();
        Ok(Debugged(slugs).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
