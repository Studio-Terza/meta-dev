use std::ffi::OsStr;

use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug)]
pub(super) struct Extensions(&'static [&'static str]);

impl Extensions {
    pub(super) fn holding(self, extension: &OsStr) -> Option<Self> {
        let lowered = extension.to_ascii_lowercase();
        let known = lowered.to_str()?;
        self.0.contains(&known).then_some(self)
    }
}
