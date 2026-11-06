use sqlx::sqlite::SqliteQueryResult;

use crate::Written;

impl Written {
    pub(crate) fn affected(done: &SqliteQueryResult) -> Written {
        Written::new(usize::try_from(done.rows_affected()).unwrap_or(usize::MAX))
    }
}
