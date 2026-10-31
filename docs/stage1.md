# Stage 1: every gap found in use gets a path

Stage 0's first evening falsified P2: 9 escapes were open, against a registered maximum of 5. Real work immediately needed transitions that stage 0 lacked. The critic's first cycle (issues #10–#14) found more. Stage 1 gives each of them a mechanical path, and makes the predictions visible against their registered numbers.

## Behaviours

Each row is a behaviour-table test through the single in-process `Call`.

### Escapes

| Command | Given | Result |
|---|---|---|
| `meta escape <kind> <reason>` | `kind` is one of the escape kinds in `self/escapes.toml`, read from the working tree or the cache of the last read from main, with no git or network | event `Escaped` with id `escape-N`; prints the id, taken inside the locked append, the open count and the repeats of that kind |
| `meta escape <kind> --pr <pr> <reason>` | `kind` names merged PRs (`outside-merge`) | event `Escaped` carrying the PR |
| `meta escape` | unknown kind; a kind that names merged PRs without `--pr`; `--pr` with any other kind; `unclassified` | refused; lists the kinds, or says which form the kind takes |
| `meta escape <kind> <reason>` | no escape kinds can be read | event `Escaped` marked `unchecked`; prints why |
| `meta escape <kind> <reason>` | the working-tree `self/escapes.toml` exists but does not parse | the cache of the last read from main is used instead; when neither parses, the kind is marked `unchecked` and the parse error is printed |
| `meta escape <kind> <reason>` | the log does not fold, or its lines do not parse | event `Escaped` all the same, as stage 0 promised; prints that the id and counts are unavailable when the lines do not parse |
| `meta resolve <escape> <task>` | the task is done | event `Resolved`; the escape no longer counts as open |
| `meta resolve` | the task is not done, or the escape is already resolved or unknown | refused, with the reason |

### Historical escapes

Some escapes record one-off bypasses that can never recur and have no fixing task, such as "agent started without a lease because meta did not exist yet". They close as historical rather than against an unrelated done task.

| Command | Given | Result |
|---|---|---|
| `meta resolve <escape> --historical <reason>` | the escape is open | event `ResolvedHistorical`; the escape no longer counts as open |
| `meta resolve <escape> --historical <reason>` | the escape is already resolved or historical, or unknown | refused, with the reason; it names `--correcting` for an escape resolved against a task |
| `meta resolve <escape> --historical <reason> --correcting` | the escape is resolved against a task | event `Corrected`; the fold treats the escape as historical; the `Resolved` record stays in the log, and `meta status` shows "resolved against task-1, corrected to historical: <reason>" |
| `meta resolve <escape> --historical <reason> --correcting` | the escape is open, historical or already corrected | refused, with the reason |
| `meta status` P2 | any | the P2 line shows the open count with the number of escapes closed as historical, corrections included, such as "P2 open escapes: 1 (6 closed as historical)"; escapes resolved against a task are not counted; display only, P2's judgement is unchanged (owner, 2026-10-30) |
| `meta status` P1 | an `outside-merge` escape names a PR and is resolved, historical or corrected | the PR still counts as recorded; closing an escape never un-records its bypass |

A log that resolves one escape twice, or corrects an escape that is not resolved against a task, does not fold.

Repeats are counted by kind, not by free text. Escape kinds are data, each with a source.

An escape whose kind is not in main's `self/escapes.toml` (a typo, or a kind since removed) is shown by `meta status` as `unchecked`, and every such escape counts under one shared key for P2's repeats, so a typo cannot dodge the bound. `unclassified` escapes keep counting by their reason.

### The lead working a task itself

| Command | Given | Result |
|---|---|---|
| `meta claim <task> --self` | the task is open | event `Claimed` by the lead, with no lease; prints the brief |
| `meta claim <task> --self` | the task is claimed by an agent | refused |
| `meta claim <task> --agent <name>` | the task is self-claimed | refused; there is no hand-over |

`meta status` counts self-claims apart from agent starts.

### Retire and defer

| Command | Given | Result |
|---|---|---|
| `meta retire <task> <reason>` | the task is open or claimed, and not linked to an open PR | event `Retired`; the task leaves the open list |
| `meta retire <task> <reason>` | the task is deferred, done or retired | refused |
| `meta defer <task> <reason>` | the task is open | event `Deferred`; listed under deferred |
| `meta undefer <task>` | the task is deferred | event `Undeferred`; open again |

### Reviews

| Command | Given | Result |
|---|---|---|
| `meta review <task> --agent <name>` | the task is linked to an open PR | event `ReviewLeased`, with a fresh lease; prints a review brief: the goal and constraints, unprimed |
| `meta verdict <task> pass\|fail` | a review lease on the task was used | event `Reviewed`; `meta status` shows the latest verdict per linked PR, at the head it was given |

No separate task is opened for a review. `meta merge` does not require a review; CI remains the only merge gate.

### Printing a brief again

A lease that is not yet used can be handed over without the lead writing the agent's instructions by hand.

| Command | Given | Result |
|---|---|---|
| `meta brief <lease>` | the lease is unused and its task is open, claimed or linked | prints the brief that the claim or review which issued the lease printed: task, agent, deadline, setup and rules, or the review goal and constraints; no event |
| `meta brief <lease>` | the lease is used | refused; the reason says each lease starts one agent, and the next step is a fresh claim or review |
| `meta brief <lease>` | the lease is unknown, or its task is done, retired or deferred, or a review lease's task no longer links its PR | refused with the reason |
| `meta claim <task> --agent <name>` | the task already has an unused work lease | refused; the next step names `meta brief <lease>` |
| `meta review <task> --agent <name>` | the task already has an unused review lease | refused; the next step names `meta brief <lease>` |

`Leased` and `ReviewLeased` carry the agent the lease is for, so the reprinted brief names the same agent. A lease recorded without one prints the task's agent (work) or "not recorded" (review). The deadline is the one the lease was issued with; the rules and origin are read from main when the brief is printed.

### Resumes

| Hook | Given | Result |
|---|---|---|
| `meta hook post-tool-use` with tool `Agent` or `Task` | the description is a used lease with no agent recorded yet | event `Started` with the lease and `tool_response.agentId` |
| `meta hook post-tool-use` with tool `Agent` or `Task` | anything else, or input that cannot be parsed | nothing recorded; `decision: block` adds the reason beside the tool result |
| `meta hook pre-tool-use` with tool `SendMessage` | `message` is exactly `continue`, `summary` is exactly the id of a used lease, `to` is the agent id recorded as started on it, and the lease's task is still open, claimed or linked | allowed; event `Resumed` against that lease |
| `meta hook pre-tool-use` with tool `SendMessage` | anything else: other message text, a missing or unknown lease, an unused one, one with no agent recorded, another agent, or a task no longer active | denied: new instructions need a fresh agent on a fresh lease |

The Agent tool takes no name that meta can set, so the PostToolUse hook binds each used lease to the agent id Claude Code reports for it. A canned `continue` restarts an agent after a failure outside its control, such as a disconnect, a crash or a timeout, without adding instructions. Resumes are not capped, because some failures are outside our control. `meta status` flags any lease from its third resume onward, so repeated failures are visible (owner, 2026-10-30). Once headless workers exist, meta sends the same canned continue itself.

### Deadlines

Agents can and will stall (owner, 2026-10-30). Every lease carries a deadline, so a stalled agent shows in `meta status` instead of only in prose.

| Command | Given | Result |
|---|---|---|
| `meta claim <task> --agent <name>` | the claim is granted | event `Leased` with `deadline`: the clock's time plus the work default in `self/roles.toml` (90 min); the brief prints "deadline: <time>, report or stop by then" |
| `meta review <task> --agent <name>` | the review is granted | event `ReviewLeased` with `deadline`: the clock's time plus the review default in `self/roles.toml` (30 min); the brief prints the deadline the same way |
| `meta claim` or `meta review` | `self/roles.toml` on main cannot be read | error; nothing is recorded |
| `meta claim <task> --agent <name> --deadline <minutes>`, `meta review <task> --agent <name> --deadline <minutes>` | `<minutes>` is a whole number of at least 1 | the deadline is the clock's time plus `<minutes>` instead of the default |
| `meta claim <task> --self` | any | no deadline; the brief prints "deadline: none"; `--deadline` with `--self` is malformed |
| `meta hook post-tool-use` with tool `Agent` or `Task` | the used lease has a deadline | event `Started` also carries `deadline`: the start time plus the lease's allowance, so the deadline counts from when the agent started |
| `meta status` | a lease's task is open, claimed or linked, its deadline has passed, and it is not a review lease with a verdict | listed under "overdue leases (flagged)", next to the resume flags, with the agent id when one is recorded, else "agent unknown" or "unused" |
| `meta status` | the task of an overdue lease is done | the lease is no longer flagged |
| `meta status` | a lease recorded before deadlines existed | it folds with no deadline: "live leases" shows "no deadline", and it is never flagged overdue |

The role defaults are data, each with a source. Headless workers will be stopped at their deadline later; for now the flag is the only enforcement.

### Waiting on the owner, and the idle alarm

Work stalled for about an hour with unblocked tasks open and nothing flagged it (owner, 2026-10-31). A task that waits on an owner decision is also not late, so it must not be flagged overdue.

| Command | Given | Result |
|---|---|---|
| `meta wait <task> --owner <question>` | the task is open, claimed or linked, and not waiting | event `WaitingOnOwner { task, question }`; the task's lease deadlines pause |
| `meta wait` | the task is unknown, deferred, done or retired, or already waiting | refused, with the reason |
| `meta unwait <task> <answer>` | the task is waiting | event `OwnerAnswered { task, answer }`; each lease the task had when it began to wait gets its deadline moved later by the time it waited, so the deadlines resume from the answer |
| `meta unwait` | the task is unknown or not waiting | refused |
| `meta status` | a task is waiting on the owner | listed under "waiting on the owner (flagged)" as `<task>: <question> (waiting N min)`, N the whole minutes since `meta wait`; its leases are not flagged overdue while it waits |
| `meta status` | a task done, retired or deferred while waiting | it is no longer listed as waiting |
| `meta status` | no task waits | "waiting on the owner (flagged)" shows "(none)" |
| `meta status` | no running work, and at least one task is stalled | "idle (flagged)" shows `idle: no live work; stalled: <task> (<why>), ...` with each stalled task by id, in task order |
| `meta status` | otherwise | "idle (flagged)" shows "(none)" |

Running work is a work lease that has a deadline, on a task an agent is working: open or claimed. A lease on a linked task is not running work, so a leftover lease on a linked task does not silence the alarm. One exception: a work lease with a deadline issued after the failing review of a linked task's pinned head is the fix under way, and counts. A lease of a task waiting on the owner is paused and does not count. A review lease is never work.

A task is stalled when it is not waiting on the owner and has no next step. This is a state check, with no age threshold:

| Task | Stalled when | Shown as |
|---|---|---|
| open | always | `open` |
| claimed, by an agent or by the lead with `--self` | always | `claimed` |
| linked | no review lease on it is live (unused, or used with no verdict yet), and its pinned head has no verdict | `no verdict on head` |
| linked | no review lease is live, the latest verdict on its pinned head is pass, and its PR is not merged | `pass on head, PR not merged` |
| linked | no review lease is live, the latest verdict on its pinned head is fail, and no running work lease was issued after that review | `fail on head, no live work lease` |
| deferred, retired, done | never | |

A verdict on an earlier head does not count for the pinned head. A linked task whose PR is merged and that awaits `done` is not stalled. The alarm needs both conditions: no running work, and N of at least 1 stalled tasks.

### Predictions against registered numbers

`self/predictions.toml` holds each measurable prediction as data. Today these are P1, P2 and P6, each quoting its `docs/prereg.md` row verbatim, with its falsifier as numbers. A falsifier names at least one measure; a prediction without one shows `UNDECIDED`.

- P1: unrecorded bypasses are the PRs merged outside meta, logged or missing from the log, that no `outside-merge` escape names. An escape of another kind, or an unclassified one, records no merge.
- P1 also judges agent starts (critic issue #11). A start that bypasses meta is one known to have happened without a lease. Meta sees that only through an escape of kind `outside-start`, which counts one unrecorded bypass each (a `Started` record without a used lease does not fold, so it shows as a corrupt log, not as a count). Meta cannot see an unleased start nobody recorded, so with no known bypass the measure is `UNDECIDED` with that reason, never `holds`. A known bypass falsifies P1 even when the merged list is unreadable.
- A hook denial is a start the hook prevented. It is shown as "prevented by the hook" and never counts against P1.
- P2: prereg's "reason" is read as "escape kind" (field `terms`). The owner's registered intent is that the third escape of one kind falsifies, so the bound is repeats above 1.

| Command | Given | Result |
|---|---|---|
| `meta status` | any | each prediction line shows its observed value, its registered expectation, and `FALSIFIED` when the falsifier holds |
| `meta status` | a measure of a prediction's falsifier is above its bound and no `Falsified` event names that prediction and measure | event `Falsified { prediction, measure, observed }`, stamped with the time of the record, appended inside the same locked append that status reads under |
| `meta status` | a `Falsified` event exists for the prediction | the line ends with `falsified at <time> (<observed>)` for each latched measure, whatever the current reading, so resolving escapes cannot erase a falsification; no new event |

| `meta status` P1 | no unrecorded merge bypass and no `outside-start` escape | `UNDECIDED; observed unrecorded bypasses unavailable (agent starts without a lease are not observable ...)`, never `holds` |
| `meta status` P1 | an `outside-start` escape exists, with or without a readable merged list | `FALSIFIED`; each such escape adds one to the unrecorded bypasses |
| `meta status` | the log holds `Denied` agent starts | the P1 starts line shows `prevented by the hook: <n>`; denials never change the P1 verdict |
| `meta status` | any | the P1 starts line reads `P1 agent starts through meta: <n>; prevented by the hook: <n>; bypassed, from outside-start escapes: <n>; self-claims, not agent starts: <n>` |

A falsification latches once per prediction and measure, so a prediction first falsified by one measure and later by another latches twice (critic issue #17). Time is shown as unix seconds.

### Rework

P6 counts CI runs per merged PR. It cannot see PRs that never merge or failed reviews (critic issue #25): red-first alone gives every PR 2 runs, and #21 has had 4 failed reviews and counts nowhere. The rework section of `meta status` shows both per task. It is report only: no prediction reads it, and `self/predictions.toml` and P6's wording are unchanged.

| Command | Given | Result |
|---|---|---|
| `meta status` | a task was ever linked to a PR, in any stage: open PR, merged, or retired after linking | an open PR is read live through the GitHub port (`gh pr view` for the branch, then `gh run list`); a merged PR is not read: its CI runs are the count recorded in its `Merged` event, because a merged PR's count cannot change; the task counts toward the total |
| `meta status` | the task has a failed review round, or CI runs above one | a line `task-N (#pr, <stage>): failed review rounds <n>, CI runs <n>`; a task with several PRs shows each and sums their runs |
| `meta status` | the CI runs of a PR cannot be read | the line shows `CI runs unavailable (<why>)` and still lists the failed rounds; it never reads as zero |
| `meta status` | any | `rework total: tasks with 2 or more failed review rounds: <n> of <m> tasks with a PR; worst: task-N (...)`; the worst case has the most failed rounds, then the most CI runs; `none` when no task has rework |

`pull` reads the branch name once and runs `gh run list` with it; `runs` shares that one call. Every port command (`gh`, `git`) is killed after a named timeout (`PORT_TIMEOUT`, 30 s) and returns a port error `<command>: timed out after <n> s`, so a hung `gh` or `git` cannot hang `meta status`; status shows such a PR as `CI runs unavailable`, never zero.

A failed review round is a `Reviewed` event with verdict `fail`. Tasks never linked to a PR are not counted. A task listed without failed rounds only has more than one CI run, which red-first makes the norm.

### Rules

- `self/rules.toml` gains the owner's red-first rule: tighten the check in the first commit; push the red commit first; waiting for its CI run is the one allowed CI wait (poll every 60 s, at most 20 min); then push the fix.
- `self/rules.toml` gains a review rule: flag any change that loosens a gate, such as the CI workflow, lint config, parse boundaries, rules, predictions or escape kinds. Source: owner 2026-10-30, ordinary review instead of blocking approvals. `meta review` puts it in every review brief.
- The push-and-stop rule names its one exception, the red-first wait.
- `self/rules.toml` gains a rule that leased agents run in the background (`run_in_background`): only background launches report an agent id at start, so a crashed foreground agent cannot be resumed. Source: failure 2026-10-30, stage 1 re-review.
- Behaviour tests use a rules fixture, not the repo's `self/rules.toml`, so changing a rule cannot break unrelated tests.

## Deferred

Blocking owner approval of gate changes waits for a proper escalation system. Blocking requests must neither flood the owner nor give agents a reason to stop work (owner, 2026-10-30). Until then, reviews catch gate loosening.

## After merge

- Resolve the bootstrap escapes against the stage 0 task.
- Resolve the stage-0 gap escapes against this stage's task.
- Read the predictions in `meta status` together with the owner.

## Timebox

2 hours of wall time from the first agent start. Overrunning means stopping and talking to the owner.
