use std::fmt;

use lint_core::{Finding, Fix};
use lint_layout::{Word, words};

use super::Reserved;

#[derive(Debug, Default)]
pub(crate) struct CountedName;

impl Reserved for CountedName {
    const OWNERS: [Word<'static>; 2] = words!["port_core", "port_derive"];
    const NAMES: &'static [Word<'static>] = &words!["counted_call", "counted_acquire"];
}

impl fmt::Display for CountedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("names counted_call or counted_acquire outside port-core and port-derive")
    }
}

impl Finding for CountedName {
    fn fix(&self) -> Fix {
        Fix::new(
            "declare the trait with #[port] and implement its Filler, so the generated code counts each call and acquisition",
        )
    }
}
