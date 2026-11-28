use newtype::Newtype;

use crate::StoreFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    statements: Vec<Statement>,
}

#[derive(Newtype, Clone, Debug, PartialEq, Eq)]
pub struct Statement(String);

pub trait Parses {
    fn parsed(&self, sql: &str) -> Result<Schema, StoreFailure>;
}

impl FromIterator<Statement> for Schema {
    fn from_iter<Statements: IntoIterator<Item = Statement>>(statements: Statements) -> Schema {
        Schema { statements: statements.into_iter().collect() }
    }
}

impl Schema {
    pub fn statements(&self) -> impl Iterator<Item = &Statement> {
        self.statements.iter()
    }
}

impl AsRef<str> for Statement {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
