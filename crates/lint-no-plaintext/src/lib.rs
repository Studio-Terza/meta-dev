mod attributes;
mod plain_fields;
mod text_mention;
mod wrappers;

use lint_core::{Headline, Lint, Report, SourceFile};

use plain_fields::PlainFields;
use wrappers::Wrappers;

#[derive(Debug)]
pub struct NoPlaintext;

impl Lint for NoPlaintext {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let wrappers = Wrappers::settled_in(&shipped);
        Report::new(
            Headline::new(
                "data that enters from a file holds no free text; split it into typed parts, or give it a newtype that validates",
            ),
            shipped.iter().flat_map(|file| PlainFields::in_file(file, &wrappers)).collect(),
        )
    }
}
