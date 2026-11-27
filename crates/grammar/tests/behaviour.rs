use fixture::{Cases, Debugged, Failed, Fixture, Spelled};
use grammar::{Bare, Fields, Ignoring, Inked, KeptLines, Letters, Lines, MigrationFile, Stamp, Unlexed};
use serde::Deserialize;

#[derive(Fixture, Deserialize, Debug)]
struct Texts {
    lines: Vec<Spelled>,
    stamps: Vec<Spelled>,
    replies: Vec<Spelled>,
    slugs: Vec<Spelled>,
    migrations: Vec<Spelled>,
    bare: Vec<Spelled>,
}

#[test]
fn every_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let drifted = Cases::<Texts>::drifted(|case| {
        let texts = case.row()?;
        let lines: Vec<_> = texts
            .lines
            .iter()
            .map(|text| {
                let lines: Vec<String> = text.as_str().parse::<Lines>().into_iter().flatten().collect();
                let kept: Vec<String> = text.as_str().parse::<KeptLines>().into_iter().flatten().collect();
                let fields: Vec<Vec<String>> = lines
                    .iter()
                    .flat_map(|line| line.parse::<Fields>().into_iter().map(|cells| cells.as_ref().to_vec()))
                    .collect();
                (text, lines, kept, fields)
            })
            .collect();
        let stamps: Vec<_> = texts
            .stamps
            .iter()
            .map(|text| (text, Stamp::try_from(text.as_str()).map(|stamp| (stamp.words(), stamp.numbers()))))
            .collect();
        let replies: Vec<_> = texts
            .replies
            .iter()
            .map(|text| (text, Inked::read(text.as_str()).map(|inked| inked.as_ref().len())))
            .collect();
        let slugs: Vec<_> = texts.slugs.iter().map(|text| (text, Letters::slugged(text.as_str()))).collect();
        let migrations: Vec<_> = texts
            .migrations
            .iter()
            .map(|text| {
                let file = text.as_str().parse::<MigrationFile>().ignoring(Unlexed::NotThisGrammar);
                (text, file.map(|read| (read.version(), read.description())))
            })
            .collect();
        let bare: Vec<_> = texts.bare.iter().map(|text| (text, Bare::word(text.as_str()))).collect();
        Ok(Debugged((lines, stamps, replies, slugs, migrations, bare)).to_string())
    })?;
    assert_eq!(drifted, Vec::new());
    Ok(())
}
