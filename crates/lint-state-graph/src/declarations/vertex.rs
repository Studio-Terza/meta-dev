use lint_core::TypeName;

#[derive(Clone, Copy, Debug)]
pub struct Vertex<'declared>(&'declared TypeName);

impl<'declared> Vertex<'declared> {
    pub(super) fn new(name: &'declared TypeName) -> Vertex<'declared> {
        Vertex(name)
    }

    pub fn name(self) -> &'declared TypeName {
        self.0
    }
}
