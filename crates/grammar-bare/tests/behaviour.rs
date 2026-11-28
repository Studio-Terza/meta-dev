use std::fs;
use std::path::Path;

use grammar_bare::Bare;
use serde::Deserialize;
use test_failure::Failed;

#[derive(Debug, Deserialize)]
struct Texts {
    bare: Vec<String>,
}

#[test]
fn the_bare_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let texts: Texts = toml::from_str(&fs::read_to_string(directory.join("texts.toml"))?)?;
    let shown: Vec<String> = texts.bare.iter().map(|text| format!("bare {text:?}: {:?}", Bare::word(text))).collect();
    let golden = directory.join("lexed.txt");
    let said = shown.join("\n");
    std::env::var_os("RECORD_GOLDENS").map_or(Ok(()), |_| fs::write(&golden, &said))?;
    assert_eq!(said, fs::read_to_string(golden)?);
    Ok(())
}
