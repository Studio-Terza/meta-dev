use design_messages::{CursorRow, CursorRowNew, StreamSchema};
use laws::proptest::prelude::{Strategy, any};
use sqlite_ports::SqliteOpener;
use store_ports::Opens;
use transcript_ports::SessionId;

laws::crud!(
    a_cursor_keeps_the_crud_laws,
    CursorRow,
    (any::<SessionId>(), any::<file_ports::Offset>()).prop_map(|(session, read_to)| CursorRowNew { session, read_to }),
    SqliteOpener.in_memory::<StreamSchema>(),
);
