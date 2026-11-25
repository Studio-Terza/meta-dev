use grammar_directive::Location;
use grammar_output::field::{Integer, Word};
use grammar_output::{Fields, Parsed, Record, Unread};

use super::{Misread, Missing};

#[derive(Debug, Record)]
struct Missed {
    #[field(Word<Missing>)]
    missing: Missing,
    #[field(Integer<_>)]
    line: usize,
    #[field(Integer<_>)]
    column: usize,
}

impl Record for Misread {
    fn read(fields: Fields) -> Result<Parsed<Misread, Fields>, Unread> {
        fields
            .record::<Missed>()?
            .concluded(|missed, rest| missed.missing.misread(Location::new(missed.line, missed.column), rest))
    }
}
