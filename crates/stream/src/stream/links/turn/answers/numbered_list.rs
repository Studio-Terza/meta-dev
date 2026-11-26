use std::cmp::Ordering;

use algebra::{Monoid, Semigroup};

use grammar_message::Number;

use crate::segment::Segment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    number: Number,
    segment: Segment,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NumberedList(Vec<Item>);

impl Item {
    pub fn top_level(segment: Segment) -> Option<Item> {
        segment.form().top_level_number().map(|number| Item { number, segment })
    }

    pub fn number(&self) -> Number {
        self.number
    }

    pub fn segment(&self) -> &Segment {
        &self.segment
    }
}

impl FromIterator<Segment> for NumberedList {
    fn from_iter<Segments: IntoIterator<Item = Segment>>(segments: Segments) -> NumberedList {
        Monoid::concat(segments.into_iter().filter_map(Item::top_level).map(|item| NumberedList(vec![item])))
    }
}

impl Semigroup for NumberedList {
    fn combine(self, later: NumberedList) -> NumberedList {
        match self.0.last().zip(later.0.first()).map(|(last, first)| last.number.cmp(&first.number)) {
            Some(Ordering::Less) | None => NumberedList(self.0.combine(later.0)),
            Some(Ordering::Equal | Ordering::Greater) => later,
        }
    }
}

impl Monoid for NumberedList {}

impl NumberedList {
    pub fn numbered(&self, number: Number) -> Option<&Item> {
        self.0.iter().find(|item| item.number == number)
    }
}
