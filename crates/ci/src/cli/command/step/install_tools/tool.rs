use std::ffi::OsStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tool(&'static str);

impl Tool {
    pub(crate) const REACH: Tool = Tool("reach");
    pub(crate) const LINT: Tool = Tool("lint");
    pub(crate) const CI: Tool = Tool("ci");
}

impl AsRef<OsStr> for Tool {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.0)
    }
}
