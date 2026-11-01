use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    name: Name,
    line: LineNumber,
}

impl Candidate {
    pub fn new(name: Name, line: LineNumber) -> Candidate {
        Candidate { name, line }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }
}
