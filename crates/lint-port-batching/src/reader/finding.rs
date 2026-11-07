use std::fmt;
use std::path::Path;

use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_layout::{LineNumber, Name, Word};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Port {
    Binding(Name),
    Field(Name),
}

#[derive(Clone, Copy, Debug)]
pub enum Within {
    Closure(Word<'static>),
    Loop(Looping),
}

#[derive(Clone, Copy, Debug)]
pub enum Looping {
    For,
    While,
    Bare,
}

#[derive(Debug)]
pub struct Finding {
    port: Port,
    within: Within,
}

impl Finding {
    pub fn new(port: Port, within: Within) -> Self {
        Self { port, within }
    }
}

impl fmt::Display for Port {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(name) => write!(formatter, "{name}"),
            Self::Field(name) => write!(formatter, "self.{name}"),
        }
    }
}

impl fmt::Display for Looping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::For => formatter.write_str("for"),
            Self::While => formatter.write_str("while"),
            Self::Bare => formatter.write_str("loop"),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.within {
            Within::Closure(adaptor) => {
                write!(
                    formatter,
                    "port {} is called inside the closure passed to {adaptor}, once per element",
                    self.port
                )
            }
            Within::Loop(looping) => {
                write!(formatter, "port {} is called inside a {looping} loop body, once per iteration", self.port)
            }
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(
            "add a batch method to the port, such as fetch_many or write_all, and call it once outside the per-element code; consuming a stream from a port is fine, and deliberate pacing is a Pacing parameter on the port method",
        )
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Spot {
    line: LineNumber,
    port: Port,
}

impl Spot {
    pub fn new(line: LineNumber, port: Port) -> Self {
        Self { line, port }
    }

    pub fn violation(self, path: &Path, within: Within) -> Violation<'_> {
        Violation::new(Site::new(path, self.line), Finding::new(self.port, within))
    }
}
