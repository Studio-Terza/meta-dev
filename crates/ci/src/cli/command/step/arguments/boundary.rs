use std::ffi::OsString;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Flag(&'static str);

impl Flag {
    pub(crate) const fn new(name: &'static str) -> Flag {
        Flag(name)
    }

    pub(super) fn names(self, argument: &OsString) -> Option<()> {
        (argument == self.0).then_some(())
    }
}
