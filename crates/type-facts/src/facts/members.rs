use crate::implementation::Implementation;
use crate::method::Method;

#[derive(Clone, Debug, Default)]
pub(crate) struct Members {
    methods: Vec<Method>,
    implementations: Vec<Implementation>,
}

impl Members {
    pub(crate) fn extend(&mut self, implementation: Option<Implementation>, methods: Vec<Method>) {
        self.methods.extend(methods);
        self.implementations.extend(implementation);
    }

    pub(crate) fn methods(&self) -> &[Method] {
        &self.methods
    }

    pub(crate) fn implementations(&self) -> &[Implementation] {
        &self.implementations
    }
}
