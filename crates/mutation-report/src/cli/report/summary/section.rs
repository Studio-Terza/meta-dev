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
}

impl<Item: fmt::Display> fmt::Display for Section<'_, Item> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\n### {} ({})\n", self.heading, Count::items_in(self.items))?;
        match self.items {
            [] => writeln!(formatter, "none"),
            items => {
                Count::SHOWN.first(items).try_for_each(|item| writeln!(formatter, "- {item}"))?;
                match Count::items_in(items).beyond(Count::SHOWN) {
                    omitted if omitted == Count::default() => Ok(()),
                    omitted => writeln!(formatter, "- ... {omitted} more in report.json"),
                }
            }
        }
    }
}
