use std::fmt;

pub(crate) struct Listed<Item, const SEPARATOR: char>(Vec<Item>);

impl<Item, const SEPARATOR: char> Listed<Item, SEPARATOR> {
    pub(crate) fn joined(items: impl IntoIterator<Item = Item>) -> Self {
        Self(items.into_iter().collect())
    }
}

impl<Item: fmt::Display, const SEPARATOR: char> fmt::Display for Listed<Item, SEPARATOR> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (head, tail) = self.0.split_at(self.0.len().min(1));
        head.iter().try_for_each(|item| write!(formatter, "{item}"))?;
        tail.iter().try_for_each(|item| write!(formatter, "{SEPARATOR} {item}"))
    }
}
