use newtype::Wrapping;

use super::Read;
use crate::{Cursor, Line, ReadFailure};

impl Read {
    pub(crate) fn complete(bytes: &[u8], cursor: Cursor) -> Result<Read, ReadFailure> {
        let consumed = bytes.iter().rposition(|byte| *byte == b'\n').map_or(0, |last| last.saturating_add(1));
        let lines = Line::parsed(bytes.get(..consumed).unwrap_or_default());
        let before = cursor.into_inner();
        let after = before.advanced(consumed).ok_or(ReadFailure::Uncounted(before))?;
        Ok(Read { lines, cursor: Cursor::new(after) })
    }
}
