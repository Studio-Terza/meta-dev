use lint_core::TypeName;

#[derive(Clone, Copy, Debug)]
pub struct Vertex<'declared>(pub(super) &'declared TypeName);

impl<'declared> Vertex<'declared> {
    pub fn name(self) -> &'declared TypeName {
        self.0
    }
}
