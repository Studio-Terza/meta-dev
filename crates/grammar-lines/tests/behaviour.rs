use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_lines::{Fields, Lines};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    lines: Vec<Spelled>,
}

#[test]
fn the_lines_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let lines: Vec<_> = case
            .row()
            .lines
            .iter()
            .map(|text| {
                let lines: Vec<String> = text.as_str().parse::<Lines>().into_iter().flatten().collect();
                let fields: Vec<Vec<String>> = lines
                    .iter()
                    .flat_map(|line| line.parse::<Fields>().into_iter().map(|cells| cells.as_ref().to_vec()))
                    .collect();
                (text, lines, fields)
            })
            .collect();
        Ok(Debugged(lines).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
