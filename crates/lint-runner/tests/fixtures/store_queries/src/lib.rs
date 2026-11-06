pub fn rows(statements: &mut Statements<'_>) -> Result<Vec<Row>, StoreFailure> {
    statements.rows(store_ports::query!(Row, "SELECT id FROM nodes"))
}
