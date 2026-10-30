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

### Predictions against registered numbers

`self/predictions.toml` holds each measurable prediction as data. Today these are P1, P2 and P6, each quoting its `docs/prereg.md` row verbatim, with its falsifier as numbers. A falsifier names at least one measure; a prediction without one shows `UNDECIDED`.

- P1: unrecorded bypasses are the PRs merged outside meta, logged or missing from the log, that no `outside-merge` escape names. An escape of another kind, or an unclassified one, records no merge.
- P2: prereg's "reason" is read as "escape kind" (field `terms`). The owner's registered intent is that the third escape of one kind falsifies, so the bound is repeats above 1.

| Command | Given | Result |
|---|---|---|
| `meta status` | any | each prediction line shows its observed value, its registered expectation, and `FALSIFIED` when the falsifier holds |
| `meta status` | a measure of a prediction's falsifier is above its bound and no `Falsified` event names that prediction and measure | event `Falsified { prediction, measure, observed }`, stamped with the time of the record, appended inside the same locked append that status reads under |
| `meta status` | a `Falsified` event exists for the prediction | the line ends with `falsified at <time> (<observed>)` for each latched measure, whatever the current reading, so resolving escapes cannot erase a falsification; no new event |

A falsification latches once per prediction and measure, so a prediction first falsified by one measure and later by another latches twice (critic issue #17). Time is shown as unix seconds.

### Rules

- `self/rules.toml` gains the owner's red-first rule: tighten the check in the first commit; push the red commit first; waiting for its CI run is the one allowed CI wait (poll every 60 s, at most 20 min); then push the fix.
- `self/rules.toml` gains a review rule: flag any change that loosens a gate, such as the CI workflow, lint config, parse boundaries, rules, predictions or escape kinds. Source: owner 2026-10-30, ordinary review instead of blocking approvals. `meta review` puts it in every review brief.
- The push-and-stop rule names its one exception, the red-first wait.
- `self/rules.toml` gains a rule that leased agents run in the background (`run_in_background`): only background launches report an agent id at start, so a crashed foreground agent cannot be resumed. Source: failure 2026-10-30, stage 1 re-review.
- Behaviour tests use a rules fixture, not the repo's `self/rules.toml`, so changing a rule cannot break unrelated tests.

### Only main builds write the real log

An agent ran `meta` built from its unmerged branch against the lead's real log. It appended events that main's binary cannot fold, so every hook denied and `meta merge` broke until the lead repaired the log by hand. The old meta-dev repo failed the same way: a stale binary against the log's schema.

The build script (`crates/meta-ports/src/real/build.rs`, with its logic in `stamp.rs`) embeds the commit the binary is built from. Edits to tracked files make the build dirty: it embeds the commit as dirty, and the refusal says how to clear it (commit or stash the tracked edits, then rebuild). Untracked files do not count. A build outside git embeds no commit. The build script reruns when any tracked file changes, or git's `HEAD`, `index` or `logs/HEAD`. It reads the tracked files from `git ls-files`, so no hand-kept list can go stale. If it cannot list them, the build is unverified.

The real log is the one in the default data dir, `$HOME/.local/share/meta-dev`. An XDG_DATA_HOME that names any other place is a scratch data dir. The exception is a place whose `meta-dev` dir, or whose `log.jsonl`, resolves through symlinks to the real one. Such a place is real. Tests use scratch or in-memory logs.

Before each write to the real log, meta asks the git port whether the embedded commit is an ancestor of origin/main. The ancestry check runs in the clone the binary was built in, by that clone's local refs, with no network. So a binary must stay in the clone it was built in. If that clone moves or is removed, every guarded write fails closed, and the refusal says that the clone has moved or been removed.

| Command | Given | Result |
|---|---|---|
| any ledger command that writes, on the real log | the build has no commit, has edits to tracked files, its commit is not on origin/main, or the ancestry cannot be read (for example, its clone is gone) | refused before any port call, with no events: "you are running a branch build (why) … Next: use a main build, or set XDG_DATA_HOME to a scratch data dir." |
| `meta hook pre-tool-use` or `post-tool-use` for `Agent`, `Task` or `SendMessage`, on the real log | the same | denied or not recorded (fail closed), with the same reason |
| `meta status`, on the real log | the same | the report prints; a falsification it would latch is not recorded, and the reason is printed |
| `meta escape <kind> <reason>`, on the real log | the same | event `Escaped` marked `build: unverified`; prints why. `meta status` shows the mark beside the escape |
| any command | a build whose commit is on origin/main, or a scratch data dir | written as before, with no mark |

`meta claim` prints how an agent gets a main build for `meta link`: a detached worktree at origin/main, built with `cargo build --release`. It also says that a branch build may run only with XDG_DATA_HOME set to a scratch dir.

## Deferred

Blocking owner approval of gate changes waits for a proper escalation system. Blocking requests must neither flood the owner nor give agents a reason to stop work (owner, 2026-10-30). Until then, reviews catch gate loosening.

## After merge

- Rebuild the lead's hook binary from main once this lands (`git pull`, then `cargo build --release`). The hook writes `LeaseUsed` and `Started`. The hook binary built from main before this change keeps writing unchecked until then; a hook binary built from a branch or a dirty tree denies every agent start.

- Resolve the bootstrap escapes against the stage 0 task.
- Resolve the stage-0 gap escapes against this stage's task.
- Read the predictions in `meta status` together with the owner.

## Timebox

2 hours of wall time from the first agent start. Overrunning means stopping and talking to the owner.
