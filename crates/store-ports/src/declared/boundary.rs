use port_core::Op;

use crate::{Argument, FromValue, Query, Statements, StoreFailure};

#[derive(Clone, Copy, Debug, Op)]
pub struct Declared {
    table: &'static str,
}

impl Declared {
    const SQL: &'static str = "SELECT sql FROM sqlite_schema WHERE tbl_name = ?1 AND sql IS NOT NULL ORDER BY name";

    #[must_use]
    pub fn of(table: &'static str) -> Declared {
        Declared { table }
    }
}

impl Query for Declared {
    type Rows = String;

    fn rows(self, statements: &mut Statements<'_>) -> Result<String, StoreFailure> {
        let rows = statements.fetched(Declared::SQL, vec![Argument::Text(self.table.to_owned())])?;
        Ok(rows
            .into_iter()
            .flatten()
            .filter_map(|value| String::from_value(value).ok())
            .collect::<Vec<String>>()
            .join("\n"))
    }
}
