use std::fmt;

use crate::cli::output::Line;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Lines<'lines>(&'lines [Line]);

impl<'lines, Listed: AsRef<[Line]>> From<&'lines Listed> for Lines<'lines> {
    fn from(listed: &'lines Listed) -> Lines<'lines> {
        Lines(listed.as_ref())
    }
}

impl fmt::Display for Lines<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|line| writeln!(formatter, "{line}"))
    }
}
