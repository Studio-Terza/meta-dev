use std::fmt;

use newtype::Wrapping;
use text_lines::OneLine;

use super::super::build::Build;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug)]
pub struct Growth<'name> {
    name: &'name OneLine,
    today: Seconds,
    average: Seconds,
}

impl<'name> Growth<'name> {
    pub(super) fn against(name: &'name OneLine, today: Seconds, earlier: &[Build]) -> Growth<'name> {
        let listed: Vec<Seconds> = earlier.iter().filter_map(|build| build.of_crate(name)).collect();
        let count = u32::try_from(listed.len().max(1)).map_or(f64::from(u32::MAX), f64::from);
        let average = Seconds::new(listed.into_iter().sum::<Seconds>().into_inner() / count);
        Growth { name, today, average }
    }

    pub(super) fn grew(self) -> Option<Growth<'name>> {
        (self.today > self.average).then_some(self)
    }

    pub(super) fn grown(&self) -> Seconds {
        Seconds::new(self.today.into_inner() - self.average.into_inner())
    }
}

impl fmt::Display for Growth<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "+{} {}: {} against {}", self.grown(), self.name, self.today, self.average)
    }
}
