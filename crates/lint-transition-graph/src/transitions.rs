mod context;
pub(crate) mod endpoint;
mod filler;
mod function;
mod ground;
mod implementer;
mod reading;
mod resolved;
mod transition;
mod wrapping;

use code_model::Model;

pub use context::Context;
pub use endpoint::Endpoint;
pub use ground::Ground;
pub use implementer::Implementer;
use reading::Reading;
pub use transition::{Inputs, Output, Transition};
pub use wrapping::Wrapping;

#[derive(Debug)]
pub struct Transitions<'file> {
    model: &'file Model,
    contexts: Vec<Context<'file>>,
}

impl<'file> From<&'file Model> for Transitions<'file> {
    fn from(model: &'file Model) -> Transitions<'file> {
        Transitions { model, contexts: Reading::from(model).contexts() }
    }
}

impl<'file> Transitions<'file> {
    #[must_use]
    pub fn contexts(&self) -> &[Context<'file>] {
        &self.contexts
    }

    #[must_use]
    pub fn model(&self) -> &'file Model {
        self.model
    }

    pub fn all(&self) -> impl Iterator<Item = Transition<'_, 'file>> {
        self.contexts
            .iter()
            .flat_map(|context| context.functions().iter().map(move |function| Transition::new(context, function)))
    }
}
