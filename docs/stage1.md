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
| `meta escape <kind> <reason>` | the log does not fold, its lines do not parse, or it is not valid UTF-8 (the standing is taken from the first line's bytes; a first line that cannot be read stays real) | event `Escaped` all the same, as stage 0 promised; prints that the id and counts are unavailable when the lines do not parse |
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
| `meta done <task> --by <pr>` | the task is open, claimed or linked, and GitHub reports the PR merged | event `Done` with `by: <pr>`; `meta status` lists the task under "done by another task's PR" as "done by #N" |
| `meta done <task> --by <pr>` | the PR is open or closed without merging, or GitHub does not report it | refused, with the reason |
| `meta done <task> --by <pr>` | the task is unknown, deferred, done or retired | refused |
| `meta done <task> --by <pr>` | `<pr>` is the task's own linked PR | refused; the next step is `meta merge` |
| `meta done <task> --by <pr>` | the task is linked and its own linked PR is open or merged, so it is not closed | refused, as `meta retire` refuses a task linked to an open PR; close the own PR first |
| `meta defer <task> <reason>` | the task is open | event `Deferred`; listed under deferred |
| `meta undefer <task>` | the task is deferred | event `Undeferred`; open again |

### Reviews

| Command | Given | Result |
|---|---|---|
| `meta review <task> --agent <name>` | the task is linked to an open PR | event `ReviewLeased`, with a fresh lease; prints a review brief: the goal and constraints, unprimed |
| `meta verdict <task> pass\|fail` | a review lease on the task was used and has no verdict, and the PR's head on GitHub is the head the lease was issued at | event `Reviewed`, with `head`: the head it judged; `meta status` shows the latest verdict per linked PR, at the head it was given |
| `meta verdict <task> pass\|fail` | the PR's head on GitHub is not the lease's head | refused; names both heads and `meta review <task> --agent <name>`; nothing is recorded |
| `meta verdict <task> pass\|fail` | the log holds a `Reviewed` whose `head` is not its lease's head | the log is corrupt; the fold fails. A `Reviewed` with no `head` (recorded before heads) folds as before |
| `meta review <task> --agent <name>` | the brief is printed | the reviewer records its own verdict (critic register U22): a constraint tells it to finish with `meta verdict <task> pass\|fail`, run with a main build, names the absolute path of the binary that printed the brief, read at run time through the git port (`current_exe`), as that main build, with no worktree or cargo step, and says to report and stop if meta refuses because the head moved |
| `meta review <task> --agent <name>` | the brief is printed | the lease line is worded for the lead, not the reviewer: "for the lead: put it in the Agent description; it works once. Reviewer: nothing for you to do with this line" |
| `meta status` | a review lease has a turn end or an end recorded with no verdict, and no later review lease of the task has a verdict | listed under "review leases with no verdict (flagged)" as `<lease> for <task>: review finished with no verdict` after a worker's end, `<lease> for <task>: review stopped by TaskStop with no verdict` after a subagent's end, or `<lease> for <task>: review's last recorded turn ended with no verdict` after only a turn end; with none, the section is not printed |

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

The role defaults are data, each with a source. A headless worker is killed at its deadline (`docs/stage2c.md`); for any other agent the flag is the only enforcement.

### Agents' turn ends and stops

Finished agents stayed flagged as overdue, and running agents were nowhere in `meta status`: "live leases" lists only unused leases (task-44). Claude Code's SubagentStop hook input has `agent_id` (hooks reference, SubagentStop: `session_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`, `stop_hook_active`, `agent_id`, `agent_type`, `agent_transcript_path`, `last_assistant_message`). meta reads only `agent_id`, the id that `Started` bound to the lease. The hook fires for Claude Code's own internal agents too; their ids match no lease, so they are ignored.

| Hook | Given | Result |
|---|---|---|
| `meta hook subagent-stop` | `agent_id` is the agent recorded by `Started` on a lease with no end recorded, at every turn end | event `Finished { lease }`, a turn end; reply `{}` |
| `meta hook subagent-stop` | `agent_id` matches no lease, or its lease has an end recorded | nothing recorded; reply `{}`, quietly |
| `meta hook post-tool-use` with tool `TaskStop` | `tool_response.task_id` is the agent recorded by `Started` on a lease with no end recorded | event `Finished { lease, end: stopped }`; reply `{}` |
| `meta hook post-tool-use` with tool `TaskStop` | `task_id` matches no lease, such as a shell task, or its lease has an end recorded | nothing recorded; reply `{}`, quietly |
| `meta hook post-tool-use` with tool `TaskStop` | a response with no readable `task_id` | nothing recorded; the reason is reported, like any other input the hook cannot parse |
| `meta hook subagent-stop` | input that cannot be parsed, or a log meta cannot read | nothing recorded; reply `{"systemMessage": <reason>}`. It never exits 2 and never sets `decision`, because either would keep the agent from stopping |
| `meta hook subagent-stop` | the stop arrives before `Started` was recorded | ignored like an unknown agent; the lease then shows no turn end recorded until its deadline, then is flagged overdue |
| `meta status` | a lease is started, no turn end or end is recorded, it has no verdict, its task is open, claimed or linked, and it is not flagged overdue | listed under "agents with no turn end or stop recorded, not flagged overdue; subagents also with no verdict, on an open task" as `<lease>: agent <agent>, <task>, role <work, review or critic>, <deadline>`; the role is critic for a task marked critic |
| `meta status` | a started lease is past its deadline and not finished | listed under "overdue leases (flagged)" only, not under "agents with no turn end or stop recorded, not flagged overdue; subagents also with no verdict, on an open task" |
| `meta status` | a lease is finished, unused, or has no agent recorded, or its task is done or retired | not listed under "agents with no turn end or stop recorded, not flagged overdue; subagents also with no verdict, on an open task" |
| `meta status` | a subagent's last record is a turn end, it has no verdict, and its task is open, claimed or linked | listed under "subagents with a turn end or stop recorded, no verdict, on an open task" as `<lease>: agent <agent>, <task>, role <role>, <deadline>, last recorded turn end <time> (N min ago), done or waiting is not known` |
| `meta status` | a subagent's last record is a TaskStop, it has no verdict, and its task is open, claimed or linked | listed under "subagents with a turn end or stop recorded, no verdict, on an open task" as `<lease>: agent <agent>, <task>, role <role>, <deadline>, stopped by TaskStop <time> (N min ago)` |
| `meta status` | a lease is finished | not listed under "overdue leases (flagged)", whatever its deadline |
| `meta status` | the only work lease is finished and a task has no next step | the idle alarm fires: a finished lease is not running work |
| canned `continue` (`Resumed`) | the lease was finished | the lease is unfinished again, so listed under "agents with no turn end or stop recorded, not flagged overdue; subagents also with no verdict, on an open task" and overdue after its deadline, until the hook records another `Finished` |
| log fold | `Finished` for a lease that is unused or has no agent recorded, or that follows a `Finished` with an end (the `Ended` state) with no resume between | corrupt log, reported like any other record that does not fit |
| log fold | `Finished` after a `Finished` with no end, with no resume between | fits. Before critic #49 any second `Finished` was corrupt. Two pairs are newly allowed: no-end after no-end, a second turn end, and with-end after no-end, a real stop after a turn end. So a doubly written turn end is no longer caught, and a build from before this change reads a log holding either pair as corrupt |

`self/claude-settings.json` adds a SubagentStop entry with no matcher: `meta hook subagent-stop || true`, timeout 10 s. Unlike the other entries it has no `onFailure: block` and no `|| exit 2`, so a missing or crashing meta lets the agent stop. The cost of a missed record is visible: the agent is listed with no turn end recorded until its deadline, then flagged overdue.

What the signals can tell (critic #49). SubagentStop fires at every turn end, not when an agent is done, so `Finished` with no `end` is a turn end. Status says so and does not say finished, for an agent can be waiting on background work that wakes it. The hook's `background_tasks` array cannot settle it: the hooks reference scopes it to the parent session, so its entries do not say whose they are. A TaskStop fires no SubagentStop. The PostToolUse entry matches `TaskStop` by name, because a matcher of plain words compares exact strings and `Task` does not match it. The log never shows a process, so neither section says running: an agent with no record can have crashed, and an agent past a turn end can have woken. Before reporting that an agent is down or live, read the process list.

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
| `meta status` | no running work, and at least one task is stalled | "idle (flagged)" shows `idle: no live work known; stalled: <task> (<why>), ...` with each stalled task by id, in task order |
| `meta status` | otherwise | "idle (flagged)" shows "(none)" |
| `meta role <task> critic` | the task exists and is not marked | event `TaskRole { task, role: critic }`; every lease on the task, past and future, is not running work, and the task is never stalled |
| `meta role <task> critic` | the task is unknown or already marked | refused; nothing is recorded |
| `meta role <task> <other>` | the role is not `critic` | usage error; nothing is recorded |

Running work is a work lease that has a deadline not yet passed at the clock's time, on a task an agent is working: open or claimed, and not marked critic. The idle alarm takes the clock for this. A lease past its deadline is not running work: the agent was to report or stop by then. A finished lease is not running work either. A lease on a linked task is not running work, so a leftover lease on a linked task does not silence the alarm. One exception: a work lease with a deadline issued after the failing review of a linked task's pinned head is the fix under way, and counts. A lease of a task waiting on the owner is paused and does not count. A review lease is never work.

The critic's task is long-running by design. The role belongs to the task, not to a claim: `meta role <task> critic` marks the task, and the mark covers every lease on it, whenever issued. A marked task is never listed as stalled. No task id is hard-coded. There is no role on `meta claim`.

Critic issue #33. On Oct 31 the alarm stayed silent at 03:06Z through a real stall: six tasks (7, 8, 12, 21, 27 open, 25 with a failed review and no fix) had no next step. The expired leases were not the cause: no work lease on those tasks was live. Only the critic's own live lease on task 14 silenced the alarm. The marking of task 14 as critic fixes it: its lease no longer counts, so the alarm fires and names the six tasks, not task 14. The regression test replays the real log cut at that point (generic titles and reasons) with the marking as its last event.

Finished leases (critic #9 U21, #33 point 4). Claude Code's SubagentStop hook tells meta when an agent's turn ends, so a lease is finished when a turn end or an end of its agent is recorded (event `Finished`) or, for a review lease, when it has its verdict. A finished lease may still have an agent waiting on background work. A finished lease is never overdue and never running work. See "Agents' turn ends and stops".

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
| `meta status` | a `Falsified` event exists for the prediction and its current reading is not `FALSIFIED` | the headline reads `FALSIFIED EARLIER; now <holds or UNDECIDED>`, then the current reading, then the latched lines; a bare `holds` never shows (critic issue #17) |
| `meta status` | the log or the side file holds escapes of kind `unclassified`, logged before kinds existed | a line `P2 unclassified escapes (from before kinds existed): <n>; they count towards repeats by their reason`; they count by reason, as before |
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

### Time and outcome per task (stage 2, spend deferred)

Stage 2 of `docs/prereg.md` measures the loop. Spend is deferred; time and outcome are built. The section is report only: no prediction reads it, and `self/predictions.toml` is unchanged. It comes from the log's timestamps and the clock, with no GitHub call, so `meta status` stays fast.

| Command | Given | Result |
|---|---|---|
| `meta status` | any task | a line `task-N: <outcome>; opened to done <n> min; waiting on the owner <n> min` for a task that ended (done or retired); `open so far <n> min` in place of `opened to done` for a task still open, claimed, linked or deferred |
| `meta status` | the task is done by its own PR | outcome `merged through meta` when the PR's `Merged` event has route `meta`, else `merged outside meta` |
| `meta status` | the task is done by `meta done --by` | outcome `done by another PR` |
| `meta status` | the task is retired, deferred, or still open, claimed or linked | outcome `retired`, `deferred` or `open` |
| `meta status` | the task waited on the owner | the wait time is the sum of its waits, each from `WaitingOnOwner` to `OwnerAnswered`, `Done`, `Retired` or `Deferred`; a wait still going counts up to now |
| `meta status` | any | a slope line across delivered tasks, a task being delivered when it is done, in the order of the `Done` records: the median minutes to deliver of the most recent 10 against the 10 before them, and the median failed review rounds per task on the same split |
| `meta status` | fewer than 10 delivered tasks before the recent 10, or none | the side with no tasks reads `none yet`; the line names how many tasks each side holds |

The slope line is the raw P4 signal. P4's wording and its falsifier are unchanged, and spend per delivered feature stays unmeasured until spend is. Time is whole minutes, rounded down. Failed review rounds are the `fail` verdicts of the rework section.

### Drift (task-48, register U1, U10, U11)

Some rules hold only by habit: every PR has a task, every PR is based on main, every commit uses the noreply identity. The drift section of `meta status` flags a breach. It is report only: it blocks nothing, adds no CI gate, and no prediction reads it. It makes one GitHub call, `gh pr list --state open --json number,baseRefName,commits --limit 49`, parsed with serde, so `meta status` stays fast.

The expected email is data: the `email` field of the `identity` rule in `self/rules.toml`. The rule's text is unchanged, and the field is read, never taken from the text. GitHub lists a commit's author first and its `Co-authored-by` trailers after, so only the first author is compared.

| Command | Given | Result |
|---|---|---|
| `meta status` | an open PR that no task is linked to | under "drift (flagged)": `#N: no linked task` |
| `meta status` | an open PR with 100 or more listed commits | `#N: commit authors unavailable: commit list may be truncated`; its commits are not compared, since `gh` lists at most 100 |
| `meta status` | an open PR whose base is not `main` | `#N: base is <base>, not main` |
| `meta status` | a commit on an open PR whose first author email is not the identity rule's `email` | `#N: commit <sha> is not by the identity email, author <email>`; a commit with no author or an empty email is flagged the same way |
| `meta status` | the identity rule has no `email` field | each open PR shows `#N: commit authors unavailable: <why>`; the other two checks still run |
| `meta status` | the open list cannot be read: `gh` fails, the reply does not parse, or it holds 49 or more PRs (a full page may be truncated, so it is never read as complete; 49 is the most GitHub's GraphQL node limit allows, since 50 PRs with 100 commits and 100 authors each is 505,050 nodes) | the section reads `drift unavailable: <why>`, never `(none)` |
| `meta status` | no PR drifts | `(none)` |

The base name and author email come from GitHub and are shown escaped on one line, so they cannot forge a status line. A PR with 100 or more commits reads as truncated, because `gh` lists at most 100 of them and an off-identity commit beyond the cap would otherwise go unseen.

### Rules

- `self/rules.toml` gains the owner's red-first rule: tighten the check in the first commit; push the red commit first; waiting for its CI run is the one allowed CI wait (poll every 60 s, at most 20 min); then push the fix.
- `self/rules.toml` gains a review rule: flag any change that loosens a gate, such as the CI workflow, lint config, parse boundaries, rules, predictions or escape kinds. Source: owner 2026-10-30, ordinary review instead of blocking approvals. `meta review` puts it in every review brief.
- The push-and-stop rule names its one exception, the red-first wait.
- `self/rules.toml` gains a rule that leased agents run in the background (`run_in_background`): only background launches report an agent id at start, so a crashed foreground agent cannot be resumed. Source: failure 2026-10-30, stage 1 re-review.
- Behaviour tests use a rules fixture, not the repo's `self/rules.toml`, so changing a rule cannot break unrelated tests.

### Only main builds write the real log

An agent ran `meta` built from its unmerged branch against the lead's real log. It appended events that main's binary cannot fold, so every hook denied and `meta merge` broke until the lead repaired the log by hand. The old meta-dev repo failed the same way: a stale binary against the log's schema.

The build script (`crates/meta-ports/src/real/build.rs`, with its logic in `stamp.rs`) embeds the commit the binary is built from. It always sets all three stamp variables, with the unused ones empty, and an empty one counts as unset, so a variable inherited from the caller cannot stand in for one left unset. Edits to tracked files make the build dirty: it embeds the commit as dirty, and the refusal says how to clear it (commit or stash the tracked edits, then rebuild). Untracked files do not count. A build outside git embeds no commit. The build script reruns when any tracked file changes, or git's `HEAD`, `index` or `logs/HEAD`. It reads the tracked files from `git ls-files`, so no hand-kept list can go stale. If it cannot list them, the build is unverified.

The real log is known by its content, not by its path. Its first record is the marker `{"at":…,"event":"RealLog"}`, written once by `meta mark-real-log`. Before every write, meta reads the first record of the log it is about to write:

- The first record is the marker: the log is real, wherever it is reached from. HOME, XDG_DATA_HOME, symlinks and hard links do not change this.
- The first record cannot be read or parsed: the log is real (fail closed).
- The log is missing or empty, or its first record is any other event: the log is scratch.
- The exception: the log in the default data dir (XDG_DATA_HOME unset, `$HOME/.local/share/meta-dev`) is real until it is marked. This keeps the lead's log guarded before its one-time marking.

Tests use scratch or in-memory logs, and fake homes.

`meta mark-real-log` puts the marker first in the log of the current data dir, ahead of the records already there. Only a main build may run it. It copies the log to `log.jsonl.unmarked` beside it first, and it refuses if that copy already exists. A log that already starts with the marker is left as it is.

Marking is crash-safe. It writes the marked log to `log.jsonl.marking` in the same directory, syncs it, renames it over the log, then syncs the directory. A crash leaves the old log or the new one, whole. If marking fails, the error names the `.unmarked` copy. The rename goes through a symlink to the file it reaches. A hard link made before marking keeps the old file: it is then a separate, unmarked file, and writes to it do not reach the real log. A process that was waiting on the log's lock during the rename opens the log again, so its append lands in the new file.

`meta status` warns while the default data dir's log is unmarked: "run meta mark-real-log with a main build".

### Escapes from unverified builds

Owner decision, 2026-10-31: a build not verified to be from main never writes an escape into the real log. It appends the escape to `escapes.unverified.jsonl`, in the directory of the file the log's path reaches. So a symlink leads to the same side file as the log's own path. A hard link cannot be followed, so its side file stands beside the link; the reply prints the path.

Each line is one JSON object with `at`, `kind`, `reason`, and optionally `pr` and `checked`. It is not a log record and has no `escape-n` id. Any build reads the file leniently: unknown fields are skipped, and a line that cannot be read is counted and is never fatal. If the file as a whole cannot be read, status says so and prints the rest of the report.

`meta status` lists each one under the open escapes as `unverified [kind] reason`. P2 counts them: they add to open escapes and to the repeats of their kind. A line below P2 gives their number and the number of unreadable lines. `meta resolve` cannot close them, because they have no id, and P1 does not read them, until `meta escape --import` moves them into the log (below). Side-file text is not trusted, so `meta status` shows it escaped on one line: line breaks, control characters, bidi and zero-width characters and the backslash appear as `\n`, `\u{1b}`, `\u{202e}`, `\\`. The P2 "most repeats" key, which can hold an unclassified reason, is shown the same way.

Before each write to the real log, meta asks the git port whether the embedded commit is an ancestor of origin/main. The ancestry check runs in the clone the binary was built in, by that clone's local refs, with no network. So a binary must stay in the clone it was built in. If that clone moves or is removed, every guarded write fails closed, and the refusal says that the clone has moved or been removed.

| Command | Given | Result |
|---|---|---|
| any ledger command that writes, on the real log | the build has no commit, has edits to tracked files, its commit is not on origin/main, or the ancestry cannot be read (for example, its clone is gone) | refused before any port call, with no events: "you are running a branch build (why) … Next: use a main build, or set XDG_DATA_HOME to a scratch data dir." |
| `meta hook pre-tool-use` or `post-tool-use` for `Agent`, `Task` or `SendMessage`, on the real log | the same | denied or not recorded (fail closed), with the same reason |
| `meta status`, on the real log | the same | the report prints; a falsification it would latch is not recorded, and the reason is printed |
| `meta escape <kind> <reason>`, on the real log | the same | no event; one line appended to `escapes.unverified.jsonl` beside the log; prints where it went and why. `meta status` shows it, marked unverified, and counts it in P2 |
| `meta escape <kind> <reason>`, on the real log | the same, and the side file cannot be written | fails, naming the side file; the log is unchanged |
| `meta escape --import`, on the real log | a main build; the side file holds readable lines | under the log's lock: one event `Escaped` per readable line, each with the next id `escape-N`, `checked` as written, and `imported { line, written }` (the line's hash and its `at`); prints each id. The records are synced to the log first; then the side file is rewritten by rename, to the lines that could not be read. `meta status` shows each as `escape-N [kind] reason (imported from an unverified build)`; P1, P2 and `meta resolve` treat it like any other escape |
| `meta escape --import` | an unreadable line in the side file | the line stays in the side file as written, so the import itself drops nothing; the reply counts them |
| `meta escape --import` | a crash after the append and before the side file is rewritten (the rewrite fails the same way) | the side file still holds every line. The next import counts, for each line, the `Escaped` records in the log that name its hash and the copies in the side file; it drops that many copies from the side file, appends nothing for them, and imports only the surplus. No escape is imported twice, and a copy the crash did not cover is still imported. Limit: byte-identical lines (same second, kind, reason, pr and checked) share one hash. A copy written after an earlier import of the same line finished looks the same as a copy left by a crash, so it is dropped as already imported and never gets an id. That needs two identical escapes in one second, one on each side of an import |
| `meta escape` (set aside) and `meta escape --import`, at the same time | a branch build appends while a main build imports | both take an exclusive lock on `escapes.unverified.lock`, beside the side file, which is never renamed. The import holds it from before it reads the side file until after it renames the rewritten one over it; an append waits, then goes to the new file. The wait is bounded like the log's (3 s). If the lock cannot be taken in time, the append fails, names the lock file and records nothing: the escape is reported as not kept, never dropped quietly. Limits: the lock is advisory (`flock`) and only meta processes take it. A branch build older than this change appends without it, and can still lose a line that lands between the import's read and its rename. A side file reached through a hard link, or on a file system without working locks, is not protected |
| `meta escape --import` | a branch build, or one whose ancestry cannot be read, on the real log | refused with the branch-build reason; neither file changes |
| `meta escape --import` | the side file is missing or empty; or a scratch log | prints that there is nothing to import; on a scratch log any build may import |
| `meta escape --import` | the log's lines do not parse | fails; the side file is untouched, because the ids cannot be known |
| `meta status` | the side file holds unknown fields or unreadable lines | the report prints; unknown fields are skipped and unreadable lines are counted |
| `meta status` | the default data dir's log is missing, empty or unmarked | the report starts with a warning: "run meta mark-real-log with a main build" |
| any command | a build whose commit is on origin/main, or a scratch log | written as before, with no mark |
| any write, on a log whose first record is the marker | a branch build reaches it from outside the default data dir: with XDG_DATA_HOME set, through a hard link or a symlink | refused with "this log is marked as the real log; use a main build", which does not advise a scratch dir (escapes go to the side file) |
| any write, on the default data dir's log with the marker | a branch build | refused as on the real log, and the refusal advises a main build or a scratch data dir |
| any write, on an unmarked log outside the default data dir | any build | written: the log is scratch |
| any write, on the default data dir's log with no marker yet | a branch build | refused as on the real log, until the log is marked |
| `meta mark-real-log` | a branch build | refused with the branch-build reason; the log is unchanged |
| `meta mark-real-log` | a main build | the marker goes first, once; a second run says the log is already marked |
| any command that writes, on the real log | `git ls-files --format` is unsupported when the binary is built | refused: "this build carries no commit (git older than 2.38 cannot stamp the build …)", not "branch build" |

`meta claim` and `meta review` print the absolute path of the binary that generated the brief, read at run time through the git port (`current_exe` in the real adapter), and tell the agent to run that exact binary for `meta link` and `meta verdict`. The lead generates briefs with a main build, so the path is a main build; no worktree or cargo build step is printed, and no machine path is committed (tests use a recorded path). The claim brief also says that a branch build may run only with XDG_DATA_HOME set to a scratch dir.

## Deferred

Blocking owner approval of gate changes waits for a proper escalation system. Blocking requests must neither flood the owner nor give agents a reason to stop work (owner, 2026-10-30). Until then, reviews catch gate loosening.

## After merge

- Rebuild the lead's hook binary from main once this lands (`git pull`, then `cargo build --release`). The hook writes `LeaseUsed`, `Started` and `Finished`; the lead also adds the SubagentStop entry of `self/claude-settings.json` to the Claude Code settings. The hook binary built from main before this change keeps writing unchecked until then; a hook binary built from a branch or a dirty tree denies every agent start.
- Then mark the lead's log once, with that main build and XDG_DATA_HOME unset: `meta mark-real-log`. It prints `marked <path> as the real log`; the log as it was stays in `~/.local/share/meta-dev/log.jsonl.unmarked`. A binary built before this change cannot read the marker, so rebuild every main build the lead uses first.

- Resolve the bootstrap escapes against the stage 0 task.
- Resolve the stage-0 gap escapes against this stage's task.
- Read the predictions in `meta status` together with the owner.

## Timebox

2 hours of wall time from the first agent start. Overrunning means stopping and talking to the owner.
