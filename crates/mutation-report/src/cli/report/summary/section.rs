use std::fmt;

use super::boundary::Count;
use super::heading::Heading;

#[derive(Debug)]
pub struct Section<'items, Item> {
    heading: Heading,
    items: &'items [Item],
}

impl<'items, Item: fmt::Display> Section<'items, Item> {
    pub(crate) fn new(heading: Heading, items: &'items [Item]) -> Section<'items, Item> {
        Section { heading, items }
    }

    fn body(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.items.first() {
            None => writeln!(formatter, "none"),
            Some(_) => Self::listed(self.items, formatter),
        }
    }

    fn listed(items: &[Item], formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Count::SHOWN.first(items).try_for_each(|item| writeln!(formatter, "- {item}"))?;
        Some(Count::items_in(items).beyond(Count::SHOWN))
            .filter(|omitted| *omitted != Count::default())
            .map_or(Ok(()), |omitted| writeln!(formatter, "- ... {omitted} more in report.json"))
    }
}

impl<Item: fmt::Display> fmt::Display for Section<'_, Item> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\n### {} ({})\n", self.heading, Count::items_in(self.items))?;
        self.body(formatter)
    }
}
