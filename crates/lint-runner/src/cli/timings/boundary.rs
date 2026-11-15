use std::any;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Spelling(&'static str);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Named(fn() -> &'static str);

impl Named {
    pub(crate) const fn spelled<Lint: ?Sized>() -> Named {
        Named(any::type_name::<Lint>)
    }

    pub(crate) fn spelling(self) -> Spelling {
        Spelling((self.0)())
    }
}

impl fmt::Display for Named {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str((self.0)())
    }
}
