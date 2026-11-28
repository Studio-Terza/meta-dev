use std::path::Path;

use grammar_letters::Letters;
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    slugs: Vec<String>,
}

#[test]
fn the_letters_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> =
        texts.slugs.iter().map(|text| format!("slug {text:?}: {:?}", Letters::slugged(text))).collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}
