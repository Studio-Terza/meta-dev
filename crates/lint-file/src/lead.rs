use lint_core::{SourceFile, Violation};
use lint_sweep::{Found, Member, Ranked, Visited};

#[derive(Debug)]
pub struct Led<'file> {
    lists: Vec<Vec<Violation<'file>>>,
    visited: Option<Visited>,
}

impl<'file> Led<'file> {
    #[must_use]
    pub fn visited(&self) -> Option<Visited> {
        self.visited
    }

    pub fn member(&mut self, member: Member) -> Vec<Violation<'file>> {
        self.lists.get_mut(member.index()).map_or_default(std::mem::take)
    }
}

impl<'file> From<Found<'file>> for Led<'file> {
    fn from(found: Found<'file>) -> Led<'file> {
        let visited = Some(found.visited());
        Led { lists: vec![found.into_iter().collect()], visited }
    }
}

impl<'file> From<Ranked<'file>> for Led<'file> {
    fn from(ranked: Ranked<'file>) -> Led<'file> {
        Led::from(Vec::from(ranked))
    }
}

impl<'file> From<Vec<Vec<Violation<'file>>>> for Led<'file> {
    fn from(lists: Vec<Vec<Violation<'file>>>) -> Led<'file> {
        Led { lists, visited: None }
    }
}

pub trait Leader: 'static {
    fn led(file: &SourceFile) -> Led<'_>;
}
