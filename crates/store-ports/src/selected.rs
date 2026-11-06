use std::marker::PhantomData;

#[derive(Debug)]
pub struct Selected<Row, Statement> {
    statement: Statement,
    row: PhantomData<Row>,
}

impl<Row, Statement> Selected<Row, Statement> {
    #[doc(hidden)]
    pub fn new(statement: Statement) -> Selected<Row, Statement> {
        Selected { statement, row: PhantomData }
    }

    pub(crate) fn statement(self) -> Statement {
        self.statement
    }
}
