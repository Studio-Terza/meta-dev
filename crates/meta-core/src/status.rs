use crate::event::Route;
use crate::ids::{Reason, RunCount};
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use crate::merges::Reconciliation;
use crate::reply::Accepted;
use std::collections::BTreeMap;

impl Ledger {
    pub fn status(&self, github: Reconciliation) -> Accepted {
        Accepted::new(Vec::new(), self.report(github))
    }

    fn report(&self, github: Reconciliation) -> String {
        let tasks: Vec<String> =
            self.tasks.values().filter(|task| !task.is_done()).map(|task| task.to_string()).collect();
        let leases: Vec<String> = self
            .leases
            .iter()
            .filter_map(|(id, slot)| match slot {
                LeaseSlot::Unused(lease) if self.live(lease) => Some(format!("{id} for {}", lease.task())),
                _ => None,
            })
            .collect();
        let reasons = self.escapes.iter().fold(BTreeMap::<&Reason, usize>::new(), |mut counts, reason| {
            *counts.entry(reason).or_default() += 1;
            counts
        });
        let escapes: Vec<String> = reasons.iter().map(|(reason, count)| format!("{count}x {reason}")).collect();
        let repeated = reasons.values().map(|count| count - 1).sum::<usize>();
        let started = self.leases.values().filter(|slot| matches!(slot, LeaseSlot::Spent(_))).count();
        let merged = self.merged.len();
        let through_meta = self.merged.iter().filter(|(_, route)| *route == Route::Meta).count();
        let outside_logged = merged - through_meta;
        let runs = RunCount::total(self.merged.iter().map(|(runs, _)| *runs));
        [
            section("open tasks", &tasks),
            section("live leases", &leases),
            section(&format!("open escapes: {}", self.escapes.len()), &escapes),
            format!("P1 agent starts through meta: {started} of {}\n", started + self.starts_denied),
            self.merges_line(through_meta, outside_logged, github),
            format!("P2 open escapes: {}; repeats of an earlier reason: {repeated}\n", self.escapes.len()),
            format!("P6 CI runs per merged PR: {runs} runs over {merged} merged PRs\n"),
        ]
        .concat()
    }
}

impl Ledger {
    fn merges_line(&self, through_meta: usize, outside_logged: usize, github: Reconciliation) -> String {
        match github {
            Reconciliation::Listed(merges) => {
                let missing = merges.prs().iter().filter(|pr| !self.landed.contains(pr)).count();
                format!(
                    "P1 merges through meta: {through_meta} of {}\nP1 merged outside meta: {} ({missing} missing from the log)\n",
                    through_meta + outside_logged + missing,
                    outside_logged + missing
                )
            }
            Reconciliation::Unavailable(why) => format!(
                "P1 merges: unavailable, GitHub merged list unreadable ({why}); logged: {through_meta} through meta, {outside_logged} outside\n"
            ),
        }
    }
}

fn section(heading: &str, lines: &[String]) -> String {
    let body = match lines.is_empty() {
        true => "  (none)\n".to_owned(),
        false => lines.iter().map(|line| format!("  {line}\n")).collect(),
    };
    format!("{heading}\n{body}")
}
