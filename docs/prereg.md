# Preregistration: the bootstrap hypothesis

Opened 2026-10-30, after the meta-dev panic. Owner decision: no thresholds are set. We monitor the measures and judge in use.

## Hypothesis

Earlier architecture proposals were sound but never took, because they stayed prose and ran on a loop that bypassed meta. Suppose three things exist first and are enforced mechanically on the lead:

- a minimal loop the lead has to use;
- an escape hatch recorded as data;
- automatic measurement.

Then proposals turn into code and stick, and the marginal cost of change stays flat.

## Bootstrap order

| Stage | What | Gates |
|---|---|---|
| 0 | Minimal loop, enforced on the lead (`docs/stage0.md`) | Lenient |
| 1 | Escapes as data; every bypass is a meta bug | Lenient |
| 2 | Measure the loop: spend, time and outcome per task | Report-only |
| 2b | Critic: a second long-lived role that names smells and reports straight to the owner (see "Critic") | Report-only |
| 3 | Rules exist only as code with a source; prose never defines a rule | Report-only at first |
| 4 | A gate becomes blocking only with measured headroom over noise and a cited source | Earned |
| 5 | Features | Normal |

## Predictions

| # | Prediction | Measure |
|---|---|---|
| P1 | The loop is used | Agent starts and merges through meta, out of all |
| P2 | Gaps get fixed, not bypassed | Open escapes, repeats per reason, and time from escape to fix |
| P3 | Rules don't drift | Normative rules that exist only as prose |
| P4 | No slough | Spend and rounds per delivered feature, as a trend |
| P5 | The lead stays on architecture | Lead share of spend spent on mechanics |
| P6 | Little rework | CI runs per merged PR |

## Critic

Owner decision, 2026-10-30. Vague, uncomfortable problems slip past automated checks by definition. Missing them cost three days, because the only channel to the owner ran through the lead, and the lead described circling as progress.

| Property | Rule |
|---|---|
| Role | Names smells, investigates them and escalates them. It never designs, plans or runs work. |
| Channel | Its findings reach the owner without passing through the lead. |
| Scope | Everything, including whether the lead's reports match the measures. |
| Briefing | Unprimed: the lead never tells it what to look for. |
| Memory | A smell register, recording for each smell: first seen, last seen, evidence, escalation count and the owner's response. A smell closes only on the owner's word or on hard evidence. An unaddressed smell escalates again. |
| Fresh eyes | Its session is replaced on a schedule. Each new instance reads the register but not the old reasoning. |
| Judged on | Smells that turned out real, not how many it raises. |
| Cost | Wakes on a schedule and on meta events, such as a measure trending the wrong way, an overrun timebox or a repeated escalation. |

**Crude version (until stage 2b):** a fresh agent on a schedule reads `meta status`, the merged and open PRs, and the lead's latest report. It files each smell as a GitHub issue labelled `critic`, which the lead does not edit, so the owner sees it directly.

| # | Prediction | Measure |
|---|---|---|
| P7 | The critic catches what the lead normalizes | Smells it raised that turned out real; time from a problem's onset to its first smell |

## Fail fast

- Any measure that trends the wrong way, or any overrun timebox, means a stop and an owner discussion, not a patch.
- A panic is always allowed.
- Rewriting from scratch is a first-class option, not a last resort.

## If a prediction fails

| Failed | Points to |
|---|---|
| P1 | The enforcement has a hole |
| P2 | Meta's design is too far from the real work |
| P4 or P6 | Agent capability or review |
| P5 | Orchestration |
