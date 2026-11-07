use std::ffi::OsStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tool(&'static str);

impl Tool {
    pub(crate) const REACH: Self = Self("reach");
    pub(crate) const LINT: Self = Self("lint");
    pub(crate) const CI: Self = Self("ci");
}

impl AsRef<OsStr> for Tool {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.0)
    }
}
