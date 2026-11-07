use lint_core::TypeName;

#[derive(Clone, Copy, Debug)]
pub struct Vertex<'declared>(&'declared TypeName);

impl<'declared> Vertex<'declared> {
    pub(super) const fn new(name: &'declared TypeName) -> Self {
        Vertex(name)
    }

    pub const fn name(self) -> &'declared TypeName {
        self.0
    }
}
