use std::path::Path;

use grammar_shell::Plain;
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    shell: Vec<String>,
}

#[test]
fn the_shell_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> =
        texts.shell.iter().map(|text| format!("shell {text:?}: {:?}", Plain::read(text))).collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}
