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
