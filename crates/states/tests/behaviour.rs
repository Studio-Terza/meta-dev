use std::cell::Cell;

use laws::proptest::prelude::any;

states::states! {
    mod pull_request {
        start Working { review -> Reviewing, abandon -> Abandoned }
        state Reviewing { approve -> Merged, rework -> Working, reject -> Abandoned }
        terminal Merged {}
        terminal Abandoned {}
    }
}

use pull_request::{Abandoned, Finished, Merged, Reviewing, Working};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Approve,
    Reject,
}

struct Flow;

impl Flow {
    fn run<'id>(pr: Working<'id>, reworks: u8, verdict: Verdict, passes: &Cell<u8>) -> Finished<'id> {
        passes.set(passes.get().saturating_add(1));
        let review: Reviewing<'id> = pr.review();
        match (reworks, verdict) {
            (0, Verdict::Approve) => Self::merged(review.approve()),
            (0, Verdict::Reject) => Self::dropped(review.reject()),
            (left, verdict) => Self::run(review.rework(), left.saturating_sub(1), verdict, passes),
        }
    }

    fn merged(pr: Merged<'_>) -> Finished<'_> {
        pr.finish()
    }

    fn dropped(pr: Abandoned<'_>) -> Finished<'_> {
        pr.finish()
    }

    fn passes(reworks: u8, verdict: Verdict) -> u8 {
        let passes = Cell::new(0);
        Working::scope(|pr| Self::run(pr, reworks, verdict, &passes));
        passes.get()
    }

    fn carried(value: u8) -> u8 {
        let seen = Cell::new(0);
        Working::scope(|pr| {
            seen.set(value);
            pr.abandon().finish()
        });
        seen.get()
    }
}

laws::law!(a_lifecycle_passes_once_per_rework_either_way, (0u8..100, any::<bool>()), input, {
    let (reworks, approved) = input;
    let verdict = if approved { Verdict::Approve } else { Verdict::Reject };
    laws::proptest::prop_assert_eq!(Flow::passes(reworks, verdict), reworks + 1);
});

laws::law!(a_scope_runs_its_body, any::<u8>(), value, {
    laws::proptest::prop_assert_eq!(Flow::carried(value), value);
});
