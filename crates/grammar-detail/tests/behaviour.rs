use std::path::Path;

use grammar_detail::Cited;
use serde::Deserialize;
use test_failure::{Cases, Failed, Golden};

#[derive(Debug, Deserialize)]
struct Texts {
    cited: Vec<String>,
}

#[test]
fn the_detail_lexer_reads_its_fixture_texts_as_its_golden_says() -> Result<(), Failed> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texts: Texts = Cases::read(&manifest.join("tests/texts.toml"))?;
    let shown: Vec<String> = texts
        .cited
        .iter()
        .map(|text| {
            let each: Vec<String> =
                Cited::each(text).iter().map(|cited| format!("{} {}", cited.project(), cited.detail())).collect();
            format!("cited {text:?}: {each:?}")
        })
        .collect();
    Golden::at(manifest, "tests/lexed.txt").checked(&shown.join("\n"))
}
