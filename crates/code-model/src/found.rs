use lint_layout::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found<Entity> {
    line: LineNumber,
    entity: Entity,
}

impl<Entity> Found<Entity> {
    #[must_use]
    pub const fn new(line: LineNumber, entity: Entity) -> Self {
        Self { line, entity }
    }

    #[must_use]
    pub const fn line(&self) -> LineNumber {
        self.line
    }

    #[must_use]
    pub const fn entity(&self) -> &Entity {
        &self.entity
    }
}
