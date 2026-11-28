use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar_notice::{TaskFile, TaskNotice};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    notices: Vec<Spelled>,
    stems: Vec<Spelled>,
}

#[test]
fn the_notice_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let texts = case.row();
        let notices: Vec<_> = texts
            .notices
            .iter()
            .map(|text| (text, TaskNotice::read(text.as_str()).map(|notice| notice.task())))
            .collect();
        let stems: Vec<_> =
            texts.stems.iter().map(|text| (text, TaskFile::read(text.as_str()).map(|file| file.ids()))).collect();
        Ok(Debugged((notices, stems)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
