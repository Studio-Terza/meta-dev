use std::fs;
use std::path::Path;

use grammar::{Bare, Fields, Ignoring, Inked, KeptLines, Letters, Lines, MigrationFile, Stamp, Unlexed};
use serde::Deserialize;
use test_failure::Failed;

#[derive(Debug, Deserialize)]
struct Texts {
    lines: Vec<String>,
    stamps: Vec<String>,
    replies: Vec<String>,
    slugs: Vec<String>,
    migrations: Vec<String>,
    bare: Vec<String>,
}

#[test]
fn every_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let texts: Texts = toml::from_str(&fs::read_to_string(directory.join("texts.toml"))?)?;
    let split = |text: &String| {
        let lines: Vec<String> = text.parse::<Lines>().into_iter().flatten().collect();
        let kept: Vec<String> = text.parse::<KeptLines>().into_iter().flatten().collect();
        let fields: Vec<Vec<String>> = lines
            .iter()
            .flat_map(|line| line.parse::<Fields>().into_iter().map(|cells| cells.as_ref().to_vec()))
            .collect();
        format!("lines {text:?}: {lines:?} {kept:?} {fields:?}")
    };
    let migrated = |text: &String| {
        let file = text.parse::<MigrationFile>().ignoring(Unlexed::NotThisGrammar);
        format!("migration {text:?}: {:?}", file.map(|read| (read.version(), read.description())))
    };
    let shown: Vec<String> = texts
        .lines
        .iter()
        .map(split)
        .chain(texts.stamps.iter().map(|text| {
            format!(
                "stamp {text:?}: {:?}",
                Stamp::try_from(text.as_str()).map(|stamp| (stamp.words(), stamp.numbers()))
            )
        }))
        .chain(
            texts
                .replies
                .iter()
                .map(|text| format!("reply {text:?}: {:?}", Inked::read(text).map(|inked| inked.as_ref().len()))),
        )
        .chain(texts.slugs.iter().map(|text| format!("slug {text:?}: {:?}", Letters::slugged(text))))
        .chain(texts.migrations.iter().map(migrated))
        .chain(texts.bare.iter().map(|text| format!("bare {text:?}: {:?}", Bare::word(text))))
        .collect();
    let golden = directory.join("lexed.txt");
    let said = shown.join("\n");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &said))?;
    assert_eq!(said, fs::read_to_string(golden)?);
    Ok(())
}
