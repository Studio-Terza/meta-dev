# Stage 1: every gap found in use gets a path

Stage 0's first evening falsified P2: 9 escapes were open, against a registered maximum of 5. Real work immediately needed transitions that stage 0 lacked. The critic's first cycle (issues #10–#14) found more. Stage 1 gives each of them a mechanical path, and makes the predictions visible against their registered numbers.

## Behaviours

Each row is a behaviour-table test through the single in-process `Call`.

### Escapes

| Command | Given | Result |
|---|---|---|
| `meta escape <kind> <reason>` | `kind` is one of the escape kinds in `self/escapes.toml` | event `Escaped` with id `escape-N`; prints the open count and the repeats of that kind |
| `meta escape` | unknown kind | refused; lists the kinds |
| `meta resolve <escape> <task>` | the task is done | event `Resolved`; the escape no longer counts as open |
| `meta resolve` | the task is not done, or the escape is already resolved or unknown | refused, with the reason |

Repeats are counted by kind, not by free text. Escape kinds are data, each with a source.

### The lead working a task itself

| Command | Given | Result |
|---|---|---|
| `meta claim <task> --self` | the task is open | event `Claimed` by the lead, with no lease; prints the brief |
| `meta claim <task> --self` | the task is claimed by an agent | refused |

`meta status` counts self-claims apart from agent starts.

### Retire and defer

| Command | Given | Result |
|---|---|---|
| `meta retire <task> <reason>` | the task is open or claimed, and not linked to an open PR | event `Retired`; the task leaves the open list |
| `meta defer <task> <reason>` | the task is open | event `Deferred`; listed under deferred |
| `meta undefer <task>` | the task is deferred | event `Undeferred`; open again |

### Reviews

| Command | Given | Result |
|---|---|---|
| `meta review <task> --agent <name>` | the task is linked to an open PR | event `ReviewLeased`, with a fresh lease; prints a review brief: the goal and constraints, unprimed |
| `meta verdict <task> pass\|fail` | a review lease on the task was used | event `Reviewed`; `meta status` shows the latest verdict per linked PR |

No separate task is opened for a review. `meta merge` does not require a review; CI remains the only merge gate.

### Resumes

| Hook | Given | Result |
|---|---|---|
| `meta hook pre-tool-use` with tool `SendMessage` | `message` is exactly `continue`, and `summary` is exactly the id of a used lease | allowed; event `Resumed` against that lease |
| `meta hook pre-tool-use` with tool `SendMessage` | anything else: other message text, a missing or unknown lease, or an unused one | denied: new instructions need a fresh agent on a fresh lease |

A canned `continue` restarts an agent after a failure outside its control, such as a disconnect, a crash or a timeout, without adding instructions. Resumes are not capped, because some failures are outside our control. `meta status` flags any lease from its third resume onward, so repeated failures are visible (owner, 2026-10-30). Once headless workers exist, meta sends the same canned continue itself.

### Predictions against registered numbers

`self/predictions.toml` holds each measurable prediction as data. Today these are P1, P2 and P6, each with the measure, the expected value and the falsifier from `docs/prereg.md`.

| Command | Given | Result |
|---|---|---|
| `meta status` | any | each prediction line shows its observed value, its registered expectation, and `FALSIFIED` when the falsifier holds |

### Gate changes

`self/gates.toml` lists the files that define gates: the CI workflow, `clippy.toml`, `self/parse-boundaries.toml`, `self/rules.toml`, `self/predictions.toml`, `self/escapes.toml`, and the lint tables in `Cargo.toml`.

| Check | Given | Result |
|---|---|---|
| CI job `gate-change` | the PR changes a gate file and has no `owner-gate-ok` label | fails, naming the files |
| CI job `gate-change` | the PR changes no gate file, or carries the label | passes |

Only the lead adds the label, and only after the owner approves in conversation. `gate-change` becomes a required check.

### Rules

- `self/rules.toml` gains the owner's red-first rule: tighten the check in the first commit, see CI fail on it, then fix.
- Behaviour tests use a rules fixture, not the repo's `self/rules.toml`, so changing a rule cannot break unrelated tests.

## After merge

- Resolve the bootstrap escapes against the stage 0 task.
- Resolve the stage-0 gap escapes against this stage's task.
- Read the predictions in `meta status` together with the owner.

## Timebox

2 hours of wall time from the first agent start. Overrunning means stopping and talking to the owner.
