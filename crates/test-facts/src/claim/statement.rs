use super::term::{Depths, Substituted, Substitution};
use super::{Accepted, Binders, Claim, Depth, Occupancy, Ranges, Term};

#[derive(Clone, Debug)]
pub(crate) struct Statement {
    binders: Binders,
    subject: Term,
    accepted: Accepted,
}

impl Statement {
    pub(crate) fn new(binders: Binders, subject: Term, accepted: Accepted) -> Statement {
        Statement { binders, subject, accepted }
    }

    pub(crate) fn restated(claim: &Claim) -> Statement {
        Statement { binders: claim.binders.clone(), subject: claim.subject.clone(), accepted: claim.accepted.clone() }
    }

    pub(crate) fn grounded(self) -> Vec<Claim> {
        let listed = self
            .binders
            .iter()
            .find_map(|(depth, binder)| binder.every().and_then(|every| every.domain().values()).map(|_| *depth));
        listed.map_or_else(|| self.clone().renumbered().claim().into_iter().collect(), |depth| self.expanded(depth))
    }

    pub(crate) fn instance(&self, depth: Depth, value: &Term) -> Statement {
        let values = Substitution::single(depth, value.clone());
        Statement {
            binders: self
                .binders
                .iter()
                .filter(|(bound, _)| **bound != depth)
                .map(|(bound, binder)| (*bound, binder.substituted(&values)))
                .collect(),
            subject: self.subject.substituted(&values),
            accepted: self.accepted.substituted(&values),
        }
    }

    pub(crate) fn restricted(mut self, depth: Depth, ranges: Ranges) -> Claim {
        self.binders.entry(depth).and_modify(|binder| *binder = binder.restricted(ranges));
        Claim { binders: self.binders, subject: self.subject, accepted: self.accepted }
    }

    fn expanded(&self, depth: Depth) -> Vec<Claim> {
        self.binders
            .get(&depth)
            .and_then(|binder| binder.domain().values())
            .into_iter()
            .flatten()
            .flat_map(|value| self.instance(depth, value).grounded())
            .collect()
    }

    fn renumbered(self) -> Renumbered {
        let undecided = self
            .binders
            .iter()
            .filter(|(_, binder)| binder.domain().occupancy() == Occupancy::Unknown)
            .map(|(depth, _)| Depths::from(*depth));
        let used: Depths =
            [self.subject.depths(), Depths::from(&self.accepted.mentions())].into_iter().chain(undecided).collect();
        let vacant = self.binders.values().find(|binder| binder.domain().occupancy() == Occupancy::Vacant).map(|_| ());
        let renumber = Substitution::renumbered(used.iter());
        let binders = used
            .iter()
            .enumerate()
            .filter_map(|(index, depth)| {
                self.binders.get(depth).map(|binder| (Depth::new(index), binder.substituted(&renumber)))
            })
            .collect();
        Renumbered {
            claim: Claim {
                binders,
                subject: self.subject.substituted(&renumber),
                accepted: self.accepted.substituted(&renumber),
            },
            vacant,
        }
    }
}

#[derive(Debug)]
struct Renumbered {
    claim: Claim,
    vacant: Option<()>,
}

impl Renumbered {
    fn claim(self) -> Option<Claim> {
        let live = self.vacant.map_or(Some(()), |()| None);
        let claim = self.claim;
        let swapped = claim
            .accepted
            .sole()
            .filter(|other| (other.closed().is_some(), *other) < (claim.subject.closed().is_some(), &claim.subject))
            .cloned();
        let oriented = swapped.map_or_else(
            || claim.clone(),
            |other| Claim {
                binders: claim.binders.clone(),
                accepted: Accepted::equal(claim.subject.clone()),
                subject: other,
            },
        );
        let trivial = oriented.accepted.sole().filter(|other| **other == oriented.subject).map(|_| ());
        let decided = oriented.subject.closed().map(|_| ());
        let unbounded = (oriented.accepted == Accepted::Any).then_some(());
        live.and(trivial.or(decided).or(unbounded).map_or(Some(oriented), |()| None))
    }
}
