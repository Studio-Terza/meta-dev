use std::fmt;

use lint_core::{Finding, Fix};
use lint_layout::{Word, words};

use super::Reserved;

#[derive(Debug, Default)]
pub(crate) struct SqlxName;

impl Reserved for SqlxName {
    const OWNERS: [Word<'static>; 2] = words!["store_ports", "store_ports_macros"];
    const NAMES: &'static [Word<'static>] = &words!["__sqlx"];
}

impl fmt::Display for SqlxName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "names __sqlx, the sqlx that store_ports re-exports for its macros, outside store-ports and store-ports-macros",
        )
    }
}

impl Finding for SqlxName {
    fn fix(&self) -> Fix {
        Fix::new("reach the store through store_ports::query!, store_ports::migrate! and the Store port, never through sqlx")
    }
}
