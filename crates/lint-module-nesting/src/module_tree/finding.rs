use std::fmt;

use super::module_path::Target;
use lint_core::Fix;

pub struct ReachesPastModRs<'path>(&'path Target<'path>);

impl<'path> From<&'path Target<'path>> for ReachesPastModRs<'path> {
    fn from(target: &'path Target<'path>) -> ReachesPastModRs<'path> {
        ReachesPastModRs(target)
    }
}

impl fmt::Display for ReachesPastModRs<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "reaches into {}", self.0)
    }
}

impl lint_core::Finding for ReachesPastModRs<'_> {
    fn fix(&self) -> Fix {
        Fix::new("import the parent's re-export instead")
    }
}
