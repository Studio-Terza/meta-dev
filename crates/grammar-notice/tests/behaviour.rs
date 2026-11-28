use std::path::Path;

use grammar_notice::{TaskFile, TaskNotice};
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    notices: Vec<String>,
    stems: Vec<String>,
}

#[test]
fn the_notice_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> = texts
        .notices
        .iter()
        .map(|text| format!("notice {text:?}: {:?}", TaskNotice::read(text).map(|notice| notice.task())))
        .chain(
            texts.stems.iter().map(|text| format!("stem {text:?}: {:?}", TaskFile::read(text).map(|file| file.ids()))),
        )
        .collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}
