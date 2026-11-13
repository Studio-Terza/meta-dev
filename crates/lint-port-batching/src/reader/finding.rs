use std::fmt;

use code_model::Repeating;
use lint_core::Fix;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Port {
    Binding(Name),
    Field(Name),
}

#[derive(Debug)]
pub(crate) struct Finding {
    port: Port,
    repeating: Repeating,
}

impl Finding {
    pub(crate) fn new(port: Port, repeating: Repeating) -> Finding {
        Finding { port, repeating }
    }
}

impl fmt::Display for Port {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Port::Binding(name) => write!(formatter, "{name}"),
            Port::Field(name) => write!(formatter, "self.{name}"),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "port {} is called inside {}", self.port, self.repeating)
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(
            "add a batch method to the port, such as fetch_many or write_all, and call it once outside the per-element code; consuming a stream from a port is fine, and deliberate pacing is a Pacing parameter on the port method",
        )
    }
}
