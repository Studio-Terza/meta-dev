use lint_core::Site;
use newtype::{Counter, Newtype};

use super::function::Function;
use super::logic::Logic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held<'file> {
    logic: Logic,
    heaviest: Site<'file>,
}

impl<'file> Held<'file> {
    pub(crate) fn summed(functions: &[Function<'file>]) -> Option<Held<'file>> {
        let heaviest = functions.iter().max_by_key(|function| function.score())?;
        let logic =
            Logic::total(functions.iter().map(|function| Logic::new(function.score().into_inner().saturating_sub(1))));
        Some(Held { logic, heaviest: AsRef::<Site<'file>>::as_ref(heaviest).clone() })
    }

    #[must_use]
    pub fn logic(&self) -> Logic {
        self.logic
    }

    #[must_use]
    pub fn heaviest(&self) -> &Site<'file> {
        &self.heaviest
    }
}
