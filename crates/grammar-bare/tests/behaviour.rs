use std::path::Path;

use grammar_bare::Bare;
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    bare: Vec<String>,
}

#[test]
fn the_bare_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> = texts.bare.iter().map(|text| format!("bare {text:?}: {:?}", Bare::word(text))).collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}
