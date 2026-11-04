use crate::implementation::Implementation;
use crate::method::Method;

#[derive(Clone, Debug, Default)]
pub(crate) struct Members<'file> {
    methods: Vec<Method>,
    implementations: Vec<Implementation<'file>>,
}

impl<'file> Members<'file> {
    pub(crate) fn extend(&mut self, implementation: Option<Implementation<'file>>, methods: Vec<Method>) {
        self.methods.extend(methods);
        self.implementations.extend(implementation);
    }

    pub(crate) fn methods(&self) -> &[Method] {
        &self.methods
    }

    pub(crate) fn implementations(&self) -> &[Implementation<'file>] {
        &self.implementations
    }
}
