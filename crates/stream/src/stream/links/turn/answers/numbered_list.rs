use std::cmp::Ordering;

use grammar::message::Number;

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
        segments.into_iter().filter_map(Item::top_level).fold(NumberedList::default(), NumberedList::continued)
    }
}

impl NumberedList {
    fn continued(self, item: Item) -> NumberedList {
        match self.0.last().map(|last| last.number.cmp(&item.number)) {
            Some(Ordering::Less) => NumberedList(self.0.into_iter().chain([item]).collect()),
            Some(Ordering::Equal | Ordering::Greater) | None => NumberedList(vec![item]),
        }
    }

    pub fn numbered(&self, number: Number) -> Option<&Item> {
        self.0.iter().find(|item| item.number == number)
    }
}
