use std::fs;
use std::path::Path;

use grammar::{Bare, Cited, Fields, KeptLines, Letters, Lines, Nested, TaskFile, TaskNotice};
use serde::Deserialize;
use test_failure::Failed;

#[derive(Debug, Deserialize)]
struct Texts {
    lines: Vec<String>,
    notices: Vec<String>,
    stems: Vec<String>,
    cited: Vec<String>,
    slugs: Vec<String>,
    bare: Vec<String>,
    shell: Vec<String>,
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
    let shown: Vec<String> = texts
        .lines
        .iter()
        .map(split)
        .chain(
            texts
                .notices
                .iter()
                .map(|text| format!("notice {text:?}: {:?}", TaskNotice::read(text).map(|notice| notice.task()))),
        )
        .chain(
            texts.stems.iter().map(|text| format!("stem {text:?}: {:?}", TaskFile::read(text).map(|file| file.ids()))),
        )
        .chain(texts.cited.iter().map(|text| {
            let each: Vec<String> =
                Cited::each(text).iter().map(|cited| format!("{} {}", cited.project(), cited.detail())).collect();
            format!("cited {text:?}: {each:?}")
        }))
        .chain(texts.slugs.iter().map(|text| format!("slug {text:?}: {:?}", Letters::slugged(text))))
        .chain(texts.bare.iter().map(|text| format!("bare {text:?}: {:?}", Bare::word(text))))
        .chain(texts.shell.iter().map(|text| {
            format!("shell {text:?}: {:?}", Nested::read(text).map(|nested| (nested.flagged(), nested.construct())))
        }))
        .collect();
    let golden = directory.join("lexed.txt");
    let said = shown.join("\n");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &said))?;
    assert_eq!(said, fs::read_to_string(golden)?);
    Ok(())
}
