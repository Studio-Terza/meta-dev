#![deny(clippy::wildcard_enum_match_arm)]

use std::path::Path;

use grammar::Grammar;
use grammar::bare::Bare;
use grammar::detail::Cited;
use grammar::letters::Letters;
use grammar::lines::{Fields, KeptLines, Lines};
use grammar::notice::{TaskFile, TaskNotice};
use grammar::shell::Plain;
use grammar::spec_line::{Attachment, Line};
use grammar::test_log::TestLine;
use proptest::prelude::{Just, Strategy, any, prop_assert_eq, prop_oneof, proptest};
use proptest::test_runner::{Config, RngSeed};
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    bare: Vec<String>,
    cited: Vec<String>,
    slugs: Vec<String>,
    lines: Vec<String>,
    notices: Vec<String>,
    stems: Vec<String>,
    shell: Vec<String>,
    logs: Vec<String>,
}

#[derive(Debug)]
struct Lexed<'texts>(&'static str, &'texts [String]);

impl Lexed<'_> {
    fn shown(&self, read: impl Fn(&str) -> String) -> impl Iterator<Item = String> {
        self.1.iter().map(move |text| format!("{} {text:?}: {}", self.0, read(text)))
    }
}

#[test]
fn every_grammar_reads_its_fixture_texts_as_the_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> = Lexed("bare", &texts.bare)
        .shown(|text| format!("{:?}", Bare::read(text).map(|_bare| text)))
        .chain(Lexed("cited", &texts.cited).shown(|text| {
            let each: Vec<String> =
                Cited::each(text).iter().map(|cited| format!("{} {}", cited.project(), cited.detail())).collect();
            format!("{each:?}")
        }))
        .chain(Lexed("slug", &texts.slugs).shown(|text| format!("{:?}", Letters::slugged(text))))
        .chain(Lexed("lines", &texts.lines).shown(|text| {
            let lines: Vec<String> = Lines::from(text).into_iter().collect();
            let kept: Vec<String> = KeptLines::from(text).into_iter().collect();
            let fields: Vec<Vec<String>> =
                lines.iter().map(|line| Fields::from(line.as_str()).as_ref().to_vec()).collect();
            format!("{lines:?} {kept:?} {fields:?}")
        }))
        .chain(
            Lexed("notice", &texts.notices)
                .shown(|text| format!("{:?}", TaskNotice::read(text).map(|notice| notice.task()))),
        )
        .chain(Lexed("stem", &texts.stems).shown(|text| format!("{:?}", TaskFile::read(text).map(|file| file.ids()))))
        .chain(Lexed("shell", &texts.shell).shown(|text| format!("{:?}", Plain::read(text))))
        .chain(Lexed("line", &texts.logs).shown(|text| format!("{:?}", TestLine::read(text))))
        .collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, rng_seed: RngSeed::Fixed(0), ..Config::default() })]
    #[test]
    fn a_spec_line_without_trailing_space_is_parsed_and_shown_back_as_written(
        text in "repair: main|spec: ( ?[!-~]{1,8})( [!-~]{1,8}){0,3}"
    ) {
        prop_assert_eq!(Line::read(&text).as_ref().map(ToString::to_string), Some(text));
    }

    #[test]
    fn a_spec_line_reads_back_from_its_rendering(
        text in prop_oneof![
            Just("repair: main".to_owned()),
            any::<Attachment>().prop_map(|attachment| format!("spec: {attachment}")),
            "[!-~]([ -~]{0,20}[!-~])?".prop_map(|value| format!("spec: {value}")),
        ]
    ) {
        prop_assert_eq!(Line::read(&text).as_ref().map(ToString::to_string), Some(text));
    }

    #[test]
    fn a_spec_line_of_one_attachment_carries_that_attachment(attachment in any::<Attachment>()) {
        let line = Line::read(&format!("spec: {attachment}"));
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
        let line = Line::read(&text);
        let again = line.as_ref().and_then(|read| Line::read(&read.to_string()));
        prop_assert_eq!(again, line);
    }
}
