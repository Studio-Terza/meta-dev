mod numbered_list;

use crate::anchor::Anchor;
use crate::{MechanicalLink, Message, Relation};
use numbered_list::{Item, NumberedList};

#[derive(Debug, Default)]
pub struct Answers {
    answers: Vec<Item>,
    asked: NumberedList,
}

impl Answers {
    pub fn between(replying: &Message, replied: &Message) -> Answers {
        Answers {
            answers: replying.segments().into_iter().filter_map(Item::top_level).collect(),
            asked: replied.segments().into_iter().collect(),
        }
    }

    pub fn links(&self) -> Vec<MechanicalLink> {
        self.answers.iter().filter_map(|answer| self.aligned(answer)).collect()
    }

    fn aligned(&self, answer: &Item) -> Option<MechanicalLink> {
        self.asked.numbered(answer.number()).map(|item| {
            MechanicalLink::new(
                Anchor::Segment(answer.segment().identity().clone()),
                Relation::Answers,
                Anchor::Segment(item.segment().identity().clone()),
            )
        })
    }
}
