use std::path::Path;

use grammar_lines::{Fields, KeptLines, Lines};
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    lines: Vec<String>,
}

#[test]
fn the_lines_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> = texts
        .lines
        .iter()
        .map(|text| {
            let lines: Vec<String> = text.parse::<Lines>().into_iter().flatten().collect();
            let kept: Vec<String> = text.parse::<KeptLines>().into_iter().flatten().collect();
            let fields: Vec<Vec<String>> = lines
                .iter()
                .flat_map(|line| line.parse::<Fields>().into_iter().map(|cells| cells.as_ref().to_vec()))
                .collect();
            format!("lines {text:?}: {lines:?} {kept:?} {fields:?}")
        })
        .collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}
