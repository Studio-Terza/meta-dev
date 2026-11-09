use lint_layout::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found<Entity> {
    line: LineNumber,
    entity: Entity,
}

impl<Entity> Found<Entity> {
    #[must_use]
    pub fn new(line: LineNumber, entity: Entity) -> Found<Entity> {
        Found { line, entity }
    }

    #[must_use]
    pub fn line(&self) -> LineNumber {
        self.line
    }

    #[must_use]
    pub fn entity(&self) -> &Entity {
        &self.entity
    }
}
