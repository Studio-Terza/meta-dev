use crate::anchor::Anchor;
use crate::segment::Segment;
use crate::{MechanicalLink, Relation};

pub struct Answers {
    answers: Vec<Segment>,
    asked: Vec<Segment>,
}

impl Answers {
    pub fn new(answers: Vec<Segment>, asked: Vec<Segment>) -> Answers {
        Answers { answers, asked }
    }

    pub fn links(&self) -> Vec<MechanicalLink> {
        self.answers.iter().filter_map(|answer| self.aligned(answer)).collect()
    }

    fn aligned(&self, answer: &Segment) -> Option<MechanicalLink> {
        answer.form().number().and_then(|_| self.asked.iter().find(|item| item.form() == answer.form())).map(|item| {
            MechanicalLink::new(
                Anchor::Segment(answer.identity().clone()),
                Relation::Answers,
                Anchor::Segment(item.identity().clone()),
            )
        })
    }
}
