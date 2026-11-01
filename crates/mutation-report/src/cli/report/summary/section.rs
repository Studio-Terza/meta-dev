use std::fmt;

use super::bits::Count;

#[derive(Debug)]
pub struct Section<'items> {
    title: &'static str,
    items: &'items [String],
}

impl<'items> Section<'items> {
    pub(crate) fn new(title: &'static str, items: &'items [String]) -> Section<'items> {
        Section { title, items }
    }
}

impl fmt::Display for Section<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\n### {} ({})\n", self.title, Count::items_in(self.items))?;
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
