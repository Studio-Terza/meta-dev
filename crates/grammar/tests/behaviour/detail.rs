use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_detail::Cited;
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    cited: Vec<Spelled>,
}

#[test]
fn the_detail_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let cited: Vec<_> = case
            .row()
            .cited
            .iter()
            .map(|text| {
                let each: Vec<_> =
                    Cited::each(text.as_str()).iter().map(|cited| (cited.project(), cited.detail())).collect();
                (text, Debugged(each).to_string())
            })
            .collect();
        Ok(Debugged(cited).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
