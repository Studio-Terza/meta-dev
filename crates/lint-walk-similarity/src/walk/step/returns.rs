use lint_layout::{Name, Whole, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Returns<'facts> {
    Itself,
    Named(&'facts Name),
}

impl<'facts> Returns<'facts> {
    const SELF: [Word<'static>; 1] = words!["Self"];

    pub(crate) fn narrowed(returned: &'facts Name, own: &Name) -> Returns<'facts> {
        if returned == own || returned.matches::<Whole>(&Self::SELF).is_some() {
            Returns::Itself
        } else {
            Returns::Named(returned)
        }
    }
}
