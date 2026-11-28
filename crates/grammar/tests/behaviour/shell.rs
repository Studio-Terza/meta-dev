use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_shell::{Edited, Nested, Unreadable};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    shell: Vec<Spelled>,
}

#[test]
fn the_shell_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let shell: Vec<_> = case
            .row()
            .shell
            .iter()
            .map(|text| {
                let edited: Vec<String> = Edited::read(text.as_str())
                    .iter()
                    .map(|edited| Debugged((edited.edit(), edited.construct(), edited.directory())).to_string())
                    .collect();
                let nested = Nested::read(text.as_str()).map(|nested| (nested.flagged(), nested.construct()));
                (text, Unreadable::read(text.as_str()), nested, edited)
            })
            .collect();
        Ok(Debugged(shell).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
