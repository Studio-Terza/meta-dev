use file_ports::Offset;
use newtype::Newtype;

use super::Read;
use crate::{Cursor, ReadFailure, Record};

impl Read {
    pub(crate) fn complete(bytes: &[u8], cursor: Cursor) -> Result<Read, ReadFailure> {
        let consumed = bytes.iter().rposition(|byte| *byte == b'\n').map_or(0, |last| last.saturating_add(1));
        let records = Record::parsed(bytes.get(..consumed).unwrap_or_default())?;
        Ok(Read { records, cursor: Self::advanced(cursor, consumed) })
    }

    fn advanced(cursor: Cursor, consumed: usize) -> Cursor {
        let before = cursor.into_inner().into_inner();
        Cursor::new(Offset::new(before.saturating_add(u64::try_from(consumed).unwrap_or(u64::MAX))))
    }
}
