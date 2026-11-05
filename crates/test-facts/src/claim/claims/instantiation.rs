use super::super::term::{Depths, Substituted, Substitution};
use super::super::{Accepted, Binder, Claim, Guard, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Entailment;

#[derive(Debug)]
pub(crate) struct View {
    guard: Guard,
    subject: Term,
    accepted: Accepted,
}

impl View {
    pub(crate) fn readings(claim: &Claim) -> Vec<View> {
        let direct =
            View { guard: claim.guard.clone(), subject: claim.subject.clone(), accepted: claim.accepted.clone() };
        let flipped = claim.accepted.sole().map(|other| View {
            guard: claim.guard.clone(),
            subject: other.clone(),
            accepted: Accepted::equal(claim.subject.clone()),
        });
        [direct].into_iter().chain(flipped).collect()
    }

    fn depths(&self) -> Depths {
        [self.subject.depths(), Depths::from(&self.accepted.mentions()), self.guard.depths()].into_iter().collect()
    }
}

#[derive(Debug)]
pub(crate) struct Instantiation<'claim> {
    pattern: &'claim Claim,
    target: &'claim Claim,
}

impl<'claim> Instantiation<'claim> {
    pub(crate) fn matching(pattern: &'claim Claim, target: &'claim Claim) -> Instantiation<'claim> {
        Instantiation { pattern, target }
    }

    pub(crate) fn entailment(&self) -> Option<Entailment> {
        View::readings(self.pattern).iter().find_map(|view| self.checked(view))
    }

    pub(crate) fn substitution(&self, view: &View) -> Option<Substitution> {
        let mut found = Substitution::default();
        view.subject.matched(&self.target.subject, &mut found)?;
        view.guard.matched(&self.target.guard, &mut found)?;
        view.accepted
            .mentions()
            .listed()
            .iter()
            .zip(self.target.accepted.mentions().listed())
            .for_each(|(pattern, target)| pattern.matched(target, &mut found).unwrap_or_default());
        view.depths().iter().try_for_each(|depth| found.bound_value(*depth).map(|_| ()))?;
        Some(found)
    }

    fn checked(&self, view: &View) -> Option<Entailment> {
        let found = self.substitution(view)?;
        (view.subject.substituted(&found) == self.target.subject).then_some(())?;
        view.accepted.substituted(&found).within(&self.target.accepted)?;
        view.guard.substituted(&found).implied_by(&self.target.guard)?;
        self.pattern
            .binders
            .iter()
            .try_for_each(|(depth, binder)| {
                found
                    .bound_value(*depth)
                    .map_or_else(|| self.unbound(binder), |value| self.bounded(binder, value))
                    .map(|_| ())
            })
            .map(|()| Entailment)
    }

    fn unbound(&self, binder: &'claim Binder) -> Option<&'claim Binder> {
        self.target.binders.values().find_map(|inner| binder.encloses(inner))
    }

    fn bounded(&self, binder: &'claim Binder, value: &Term) -> Option<&'claim Binder> {
        value.bound().map_or_else(
            || value.closed().and_then(|closed| binder.admits(closed)),
            |depth| self.target.binders.get(&depth).and_then(|inner| binder.encloses(inner)),
        )
    }
}
