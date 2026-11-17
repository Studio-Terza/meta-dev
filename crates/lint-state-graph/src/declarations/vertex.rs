use lint_core::TypeName;
use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub struct Vertex<'declared>(&'declared TypeName);

impl<'declared> Vertex<'declared> {
    pub fn name(self) -> &'declared TypeName {
        self.0
    }
}
