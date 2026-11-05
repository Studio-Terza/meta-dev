use super::super::statement::Statement;
use super::super::{Claim, Depth, Ranges, Term};
use super::instantiation::{Entailment, Instantiation, View};

#[derive(Debug)]
struct UniversalPart(Ranges);

#[derive(Debug)]
struct PointPart(Ranges);

#[derive(Debug)]
pub(crate) struct Cover<'claim> {
    before: &'claim Claim,
    depth: Depth,
    ranges: &'claim Ranges,
}

impl<'claim> Cover<'claim> {
    pub(crate) fn ranged(before: &'claim Claim) -> Vec<Cover<'claim>> {
        before
            .binders
            .iter()
            .filter_map(|(depth, binder)| {
                binder.every().and_then(|every| every.domain().ranges()).map(|ranges| Cover {
                    before,
                    depth: *depth,
                    ranges,
                })
            })
            .collect()
    }

    pub(crate) fn covered_by(&self, pool: &[Claim]) -> Option<Entailment> {
        let covered =
            pool.iter().filter_map(|after| self.part(after)).fold(Ranges::default(), |all, part| &all | &part);
        covered.encloses(self.ranges).map(|_| Entailment)
    }

    fn part(&self, after: &Claim) -> Option<Ranges> {
        self.universal(after).map(|part| part.0).or_else(|| self.point(after).map(|part| part.0))
    }

    fn universal(&self, after: &Claim) -> Option<UniversalPart> {
        let instantiation = Instantiation::matching(after, self.before);
        let found = View::readings(after).iter().find_map(|view| instantiation.substitution(view))?;
        let marker = Term::Bound(self.depth);
        let covering = after
            .binders
            .iter()
            .find(|(bound, _)| found.bound_value(**bound) == Some(&marker))
            .and_then(|(_, binder)| binder.domain().ranges())?;
        let part = covering & self.ranges;
        let restricted = Statement::restated(self.before).restricted(self.depth, &part);
        Instantiation::matching(after, &restricted).entailment().map(|_| UniversalPart(part))
    }

    fn point(&self, after: &Claim) -> Option<PointPart> {
        let instantiation = Instantiation::matching(self.before, after);
        let found = View::readings(self.before).iter().find_map(|view| instantiation.substitution(view))?;
        let point = found.bound_value(self.depth).and_then(Term::number)?;
        let instances = Statement::restated(self.before).instance(self.depth, &Term::Number(point)).grounded();
        instances
            .iter()
            .all(|instance| Instantiation::matching(after, instance).entailment().is_some())
            .then(|| PointPart(Ranges::between(point, point)))
    }
}
