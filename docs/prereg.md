# Preregistration: the bootstrap hypothesis

Opened 2026-10-30, after the meta-dev panic. Owner decision: no goals are set, but every prediction states the number we expect to observe if its hypothesis holds. A prediction is judged only against the number it was registered with.

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
| 2 | Measure the loop: spend, time and outcome per task (time and outcome built; spend deferred) | Report-only |
| 2b | Critic: a second long-lived role that names smells and reports straight to the owner (see "Critic") | Report-only |
| 2c | Orchestration: a task state machine in meta, stateless stage workers and project agents (see "Agents") | Lenient |
| 3 | Rules exist only as code with a source; prose never defines a rule | Report-only at first |
| 4 | A gate becomes blocking only with measured headroom over noise and a cited source | Earned |
| 5 | Features | Normal |

## Predictions

| # | Hypothesis | Measure | Expected if it holds | Falsified by |
|---|---|---|---|---|
| P1 | The loop is used | Agent starts and merges through meta, out of all | 100%, except recorded escapes | Any unrecorded bypass |
| P2 | Gaps get fixed, not bypassed | Open escapes; repeats per reason | Each reason fixed before its 3rd repeat; at most 5 open | A reason repeated 3 times, or more than 5 open |
| P3 | Rules don't drift | Normative rules that exist only as prose | 0 after stage 3 | Any |
| P4 | No slough | Spend per delivered feature | Within 1.5× the running median | 3 rises in a row |
| P5 | The lead stays on architecture | Lead share of spend on mechanics | At most 10% after stage 2c | Above 25% after stage 2c |
| P6 | Little rework | CI runs per merged PR | Median at most 2 | Median above 3 |
| P7 | The critic catches what the lead normalizes (see "Critic") | Time from a problem's onset to its first smell | Within 1 day | A real problem first named by the owner or lead after 2 days |
| P8 | Continuous main inflow keeps conflicts cheap (see "Project branches") | Conflict-fix spend per main inflow, against a typical worker run | Median below 5% | Above 15% |
| P9 | Distance to landable falls on a healthy project | Main gates failing on the project branch tip | Falls by at least 1 per working day | Flat for 2 days |
| P10 | Partial landings keep project branches young | Days between landings | At most 3 | 5 with no landing |
| P11 | Project branches cost less CI than slices straight to main | CI runs per delivered feature, against P6's rate | At least 30% lower | Not lower |
| P12 | Landing PRs are no harder than ordinary ones | CI runs per landing PR | At most P6's median | 2× P6's median |
| P13 | Scoped workers keep project branches green | Share of a project branch's life spent red | At most 10% | Above 25% |

## Agents

Owner decisions, 2026-10-30. An agent lives as long as the context it has to hold together.

| Kind | Holds | Lives for | Count |
|---|---|---|---|
| Stage worker | One stage of one task: spec, diff, log | That stage only | As many as the pool allows |
| Project agent | One project's design intent | Until the project is delivered; replaced from its state in meta when its context budget runs out | One per active project, in any repo |
| Lead | The owner's intent, cross-project architecture, priorities | A role, not a session; refreshed from meta's state | One |
| Critic | Smells | A role; fresh eyes on a schedule | One |

**Lead (core only):**
- talks with the owner, and records the owner's decisions;
- writes status reports across projects;
- decides what goes to the owner;
- owns cross-project architecture and project boundaries;
- sets order across projects;
- diagnoses failures across projects;
- makes the final call on architecture-level review findings;
- sets the rules and their sources;
- defines the measures.

**Project agent:**
- splits its project into tasks and owns the project design;
- sets the model tier per task and the task-specific part of each brief;
- decides the next step after each stage;
- triages review findings;
- handles CI failures on its own PRs;
- watches for circling inside its project.

It does no mechanics and never spawns agents; it asks meta for stage workers. It escalates to the lead only what crosses projects or goes against the owner's intent.

**Meta (code, not agents):**
- launches, briefs, gives deadlines to and stops stage workers;
- runs reviews as a stage;
- handles merge, update-branch, close-out and rebuilds;
- detects and retries GitHub failures;
- monitors, cleans up and gc's;
- computes the measures;
- installs hooks.

When something new comes up, meta sends it to a short-lived diagnostic worker, not to the lead.

**What reaches the lead:** escalations only, each as a short structured summary. Raw agent reports, CI logs and merge chatter never do.

**Headless workers (owner, 2026-10-30):** meta starts Claude sessions itself, as headless workers running locally, one per lease. Rules:

| Rule | Value |
|---|---|
| Concurrency | At most 4 workers at once |
| When they run | Any time, including while the owner is away |
| Rollout | Fixers and reviewers first; implementers and project agents only after data from that phase |
| Tools | An explicit allowlist. Agent, Task and starting `claude` are denied. Secrets and paths outside the worktree are denied. |
| Supervision | meta runs each worker as a process group with a deadline. If meta dies, its workers die. |
| Caps | A round cap and a token budget per task, plus `meta stop` as a kill switch. Hitting a cap escalates; nothing retries silently. |
| Visibility | `meta status` lists live workers. Transcripts are kept for each run. |

No daily spend cap is set. Spend is monitored, in line with the no-thresholds decision.

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

Prediction: P7.

## Project branches

Owner decisions, 2026-10-30. PR orchestration follows the same three levels as agents.

| Level | Agent | Branch | Lives for |
|---|---|---|---|
| Long | Lead | `main` | Always |
| Medium | Project agent | `project/<id>` | Until the project has landed |
| Short | Stage worker | Worker PR | One stage |

| Rule | Decision |
|---|---|
| Accumulation | Slices accumulate on the project branch. Main's full gates judge the end state when it lands. |
| Main inflow | Meta merges main into every project branch automatically on each change to main. A conflict starts a conflict-fix worker. |
| Partial landing | Allowed. Any coherent subset may land early. |
| Ownership | Only the owning project's workers open PRs against `project/<id>`. Only meta merges into it, on the project agent's verdict. Only `meta merge` lands it on main. |
| Distance to landable | Main's full gates run report-only on the project branch tip, continuously. |
| No stacks | Worker PRs sit flat against the project branch. |
| Outside projects | Small independent fixes go straight to main through `meta merge`. |
| Gates on project branches | Open; to be discussed separately. |

**The trap to dodge:** a long-lived integration branch that drifts from main, hides breakage and never lands, which is the slough again one level down. The hypotheses for dodging it are P8 to P13. A distance to landable that stays flat is also a critic smell.

## Deferred

Owner decision, 2026-10-30: deferred together with the rest of token observability and budgeting:
- the per-task round cap;
- the per-task token budget.

## Fail fast

- Any falsified prediction, or any overrun timebox, means a stop and an owner discussion, not a patch.
- A panic is always allowed.
- Rewriting from scratch is a first-class option, not a last resort.

## If a prediction fails

| Failed | Points to |
|---|---|
| P1 | The enforcement has a hole |
| P2 | Meta's design is too far from the real work |
| P4 or P6 | Agent capability or review |
| P5 | Orchestration |
| P7 | The critic's independence or scope |
| P8 to P13 | Project branches are drifting into the integration-branch trap |
