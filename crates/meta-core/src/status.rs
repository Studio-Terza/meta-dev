use crate::event::Attempt;
use crate::ids::{Reason, RunCount};
use crate::lease::LeaseSlot;
use crate::ledger::Ledger;
use std::collections::BTreeMap;

impl Ledger {
    pub fn status(&self) -> String {
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
        let denied = |attempt: Attempt| self.denied.iter().filter(|denied| **denied == attempt).count();
        let (starts_denied, merges_denied) = (denied(Attempt::AgentStart), denied(Attempt::Merge));
        let merged = self.merged.len();
        let runs = RunCount::total(self.merged.iter().copied());
        [
            section("open tasks", &tasks),
            section("live leases", &leases),
            section(&format!("open escapes: {}", self.escapes.len()), &escapes),
            format!("P1 agent starts through meta: {started} of {}\n", started + starts_denied),
            format!("P1 merges through meta: {merged} of {}\n", merged + merges_denied),
            format!("P2 open escapes: {}; repeats of an earlier reason: {repeated}\n", self.escapes.len()),
            format!("P6 CI runs per merged PR: {runs} runs over {merged} merged PRs\n"),
        ]
        .concat()
    }
}

fn section(heading: &str, lines: &[String]) -> String {
    let body = match lines.is_empty() {
        true => "  (none)\n".to_owned(),
        false => lines.iter().map(|line| format!("  {line}\n")).collect(),
    };
    format!("{heading}\n{body}")
}
