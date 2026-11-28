use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_bare::Bare;
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    bare: Vec<Spelled>,
}

#[test]
fn the_bare_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let bare: Vec<_> = case.row().bare.iter().map(|text| (text, Bare::word(text.as_str()))).collect();
        Ok(Debugged(bare).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
