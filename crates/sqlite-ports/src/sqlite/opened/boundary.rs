use rusqlite::{Connection, MAIN_DB};

#[derive(Clone, Debug)]
pub(in super::super) struct Snapshot(Vec<u8>);

impl Snapshot {
    pub(in super::super) fn taken(connection: &Connection) -> rusqlite::Result<Snapshot> {
        connection.serialize(MAIN_DB).map(|data| Snapshot(data.to_vec()))
    }

    pub(in super::super) fn restored(&self, connection: &mut Connection) -> rusqlite::Result<()> {
        connection.deserialize_read_exact(MAIN_DB, self.0.as_slice(), self.0.len(), false)
    }
}
