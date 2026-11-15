use std::any;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Named(fn() -> &'static str);

impl Named {
    pub(crate) const fn spelled<Lint: ?Sized>() -> Named {
        Named(any::type_name::<Lint>)
    }
}

impl fmt::Display for Named {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str((self.0)())
    }
}
