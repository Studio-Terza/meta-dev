use rusqlite::params_from_iter;
use rusqlite::types::Value as Stored;
use store_ports::{Argument, Bind, Done, Execute, StoreFailure, Value};

use super::super::super::failed::Unexecuted;
use super::{Bound, Connected};

impl Bind for Bound {
    fn null(&mut self) -> Result<(), StoreFailure> {
        self.0.push(Stored::Null);
        Ok(())
    }

    fn integer(&mut self, integer: i64) -> Result<(), StoreFailure> {
        self.0.push(Stored::Integer(integer));
        Ok(())
    }

    fn text(&mut self, text: String) -> Result<(), StoreFailure> {
        self.0.push(Stored::Text(text));
        Ok(())
    }
}

impl Execute for Connected<'_> {
    fn fetched(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Vec<Vec<Value>>, StoreFailure> {
        let mut statement = self.connection.prepare_cached(sql).map_err(Unexecuted::from)?;
        let rows = statement
            .query_map(params_from_iter(Bound::arguments(arguments)?), Connected::row)
            .map_err(Unexecuted::from)?;
        rows.map(|row| row.map_err(Unexecuted::from).map_err(StoreFailure::from).and_then(|values| values)).collect()
    }

    fn executed<D: Done>(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<D, StoreFailure> {
        let mut statement = self.connection.prepare_cached(sql).map_err(Unexecuted::from)?;
        let changed = statement.execute(params_from_iter(Bound::arguments(arguments)?)).map_err(Unexecuted::from)?;
        Ok(D::changed(changed))
    }
}
