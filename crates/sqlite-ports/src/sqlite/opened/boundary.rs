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

#[derive(Debug)]
pub(in super::super) struct Emptying {
    script: String,
}

impl Emptying {
    const LISTED: &'static str = "
SELECT
  '\"' || replace(listed.name, '\"', '\"\"') || '\"'
FROM
  pragma_table_list AS listed
  LEFT JOIN sqlite_schema AS declared ON declared.name = listed.name
WHERE
  listed.schema = 'main'
  AND listed.name NOT IN ('sqlite_schema', 'sqlite_temp_schema', '_sqlx_migrations')
  AND CASE ?1
    WHEN 'plain' THEN listed.type = 'table'
    WHEN 'indexed' THEN listed.type = 'virtual' AND lower(declared.sql) LIKE '%using fts5%'
    ELSE listed.type = 'virtual' AND lower(declared.sql) NOT LIKE '%using fts5%'
  END
";
    const PLAIN: &'static str = "plain";
    const INDEXED: &'static str = "indexed";
    const FOREIGN: &'static str = "foreign";

    pub(in super::super) fn listed(connection: &Connection) -> rusqlite::Result<Option<Emptying>> {
        let foreign = Self::named(connection, Self::FOREIGN)?;
        let emptying = Emptying {
            script: Self::script(&Self::named(connection, Self::PLAIN)?, &Self::named(connection, Self::INDEXED)?),
        };
        Ok(foreign.is_empty().then_some(emptying))
    }

    pub(in super::super) fn emptied(&self, connection: &Connection) -> rusqlite::Result<Option<()>> {
        connection.is_autocommit().then(|| connection.execute_batch(&self.script)).transpose()
    }

    fn named(connection: &Connection, kind: &str) -> rusqlite::Result<Vec<String>> {
        let mut statement = connection.prepare(Self::LISTED)?;
        let rows = statement.query_map([kind], |row| row.get(0))?;
        rows.collect()
    }

    fn script(plain: &[String], indexed: &[String]) -> String {
        let cleared = plain.iter().map(|quoted| format!("DELETE FROM {quoted};"));
        let rebuilt = indexed.iter().map(|quoted| format!("INSERT INTO {quoted} ({quoted}) VALUES ('rebuild');"));
        let steps: Vec<String> = cleared.chain(rebuilt).collect();
        format!("PRAGMA foreign_keys = OFF;\nBEGIN;\n{}\nCOMMIT;\nPRAGMA foreign_keys = ON;", steps.join("\n"))
    }
}
