use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar::{Bare, Cited, Edited, Fields, Letters, Lines, Nested, TaskFile, TaskNotice};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    lines: Vec<Spelled>,
    notices: Vec<Spelled>,
    stems: Vec<Spelled>,
    cited: Vec<Spelled>,
    slugs: Vec<Spelled>,
    bare: Vec<Spelled>,
    shell: Vec<Spelled>,
}

#[test]
fn every_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let texts = case.row();
        let lines: Vec<_> = texts
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
        let notices: Vec<_> = texts
            .notices
            .iter()
            .map(|text| (text, TaskNotice::read(text.as_str()).map(|notice| notice.task())))
            .collect();
        let stems: Vec<_> =
            texts.stems.iter().map(|text| (text, TaskFile::read(text.as_str()).map(|file| file.ids()))).collect();
        let cited: Vec<_> = texts
            .cited
            .iter()
            .map(|text| {
                let each: Vec<_> =
                    Cited::each(text.as_str()).iter().map(|cited| (cited.project(), cited.detail())).collect();
                (text, Debugged(each).to_string())
            })
            .collect();
        let slugs: Vec<_> = texts.slugs.iter().map(|text| (text, Letters::slugged(text.as_str()))).collect();
        let bare: Vec<_> = texts.bare.iter().map(|text| (text, Bare::word(text.as_str()))).collect();
        let shell: Vec<_> = texts
            .shell
            .iter()
            .map(|text| {
                let edited: Vec<String> = Edited::read(text.as_str())
                    .iter()
                    .map(|edited| Debugged((edited.edit(), edited.construct(), edited.directory())).to_string())
                    .collect();
                (text, Nested::read(text.as_str()).map(|nested| (nested.flagged(), nested.construct())), edited)
            })
            .collect();
        Ok(Debugged((lines, notices, stems, cited, slugs, bare, shell)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
