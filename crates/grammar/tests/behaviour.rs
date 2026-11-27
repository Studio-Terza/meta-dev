use std::fs;
use std::path::Path;

use grammar::{
    Attachment, Bare, Fields, Ignoring, Inked, KeptLines, Letters, Line, Lines, MigrationFile, Stamp, Unlexed,
};
use proptest::prelude::{Just, Strategy, any, prop_assert_eq, prop_oneof, proptest};
use proptest::test_runner::{Config, RngSeed};
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

struct Spec;

impl Spec {
    fn read(text: &str) -> Option<Line> {
        text.parse::<Line>().ignoring(Unlexed::NotThisGrammar)
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_spec_line_without_trailing_space_is_parsed_and_shown_back_as_written(
        text in "repair: main|spec: ( ?[!-~]{1,8})( [!-~]{1,8}){0,3}"
    ) {
        prop_assert_eq!(Spec::read(&text).as_ref().map(ToString::to_string), Some(text));
    }

    #[test]
    fn a_spec_line_reads_back_from_its_rendering(
        text in prop_oneof![
            Just("repair: main".to_owned()),
            any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
            "[!-~]([ -~]{0,20}[!-~])?".prop_map(|value| format!("spec: {value}")),
        ]
    ) {
        prop_assert_eq!(Spec::read(&text).as_ref().map(ToString::to_string), Some(text));
    }

    #[test]
    fn a_spec_line_of_one_attachment_carries_that_attachment(attachment in any::<Attachment>()) {
        let line = Spec::read(&format!("spec: {attachment}"));
        prop_assert_eq!(line.as_ref().and_then(Line::spec).and_then(|spec| spec.attachment()), Some(&attachment));
    }

    #[test]
    fn any_text_the_spec_line_lexer_accepts_reads_back_from_its_rendering(
        text in prop_oneof![
            any::<String>(),
            "(spec: |repair: main)?[a-z0-9#, \t\r\n-]{0,12}",
            any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
        ]
    ) {
        let line = Spec::read(&text);
        let again = line.as_ref().and_then(|read| Spec::read(&read.to_string()));
        prop_assert_eq!(again, line);
    }
}
