use code_model::{Position, Reach, Within};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listing {
    Blocking,
    Widened,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Legacy {
    Eligible,
    Ineligible,
}

impl Legacy {
    pub fn both(self, other: Legacy) -> Legacy {
        match self {
            Legacy::Eligible => other,
            Legacy::Ineligible => Legacy::Ineligible,
        }
    }

    pub fn of_reach(reach: Reach) -> Legacy {
        match reach {
            Reach::Shipped => Legacy::Eligible,
            Reach::Test | Reach::MacroBody => Legacy::Ineligible,
        }
    }

    pub fn of_within(within: Within) -> Legacy {
        match within {
            Within::Own => Legacy::Eligible,
            Within::TraitImpl => Legacy::Ineligible,
        }
    }

    pub fn of_position(position: Position) -> Legacy {
        match position {
            Position::Expression => Legacy::Eligible,
            Position::Statement => Legacy::Ineligible,
        }
    }

    pub fn unless<Found>(found: Option<&Found>) -> Legacy {
        match found {
            Some(_) => Legacy::Ineligible,
            None => Legacy::Eligible,
        }
    }

    pub fn listing(self) -> Listing {
        match self {
            Legacy::Eligible => Listing::Blocking,
            Legacy::Ineligible => Listing::Widened,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Exempting<Fact> {
    fact: Fact,
    reach: Reach,
}

impl<Fact> Exempting<Fact> {
    pub fn new(fact: Fact, reach: Reach) -> Exempting<Fact> {
        Exempting { fact, reach }
    }

    pub fn covering(&self, finding: Reach) -> Option<&Fact> {
        match self.reach {
            Reach::Shipped => Some(&self.fact),
            Reach::Test | Reach::MacroBody => {
                [finding].into_iter().find(|finding| *finding == self.reach).map(|_| &self.fact)
            }
        }
    }
}
