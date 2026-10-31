# Owner role as data

Draft for task-61. The owner has not approved it. Facts are as of main 79da067, 2026-10-31.

The owner, 2026-10-29: "we have de facto three lists of items for discussion with me: 1. items which are completely blocking (ie, work stalls) 2. items which are immediate but nonblocking 3. items which are deferred. ... what specifically does a blocking discussion item block? usually it's a clearly definable subset of something."

Later limits: "I don't want to get bombarded by a bunch of blocking requests, or to have agents use blocking requests as an excuse to stop work." "stalls should never happen". "gh should enforce that, not anything local". "anything we want doesn't exist unless it's enforced somehow".

This is the "proper escalation system" that `docs/stage1.md`, "Deferred", waits for.

## What the Oct 29 model needs changed

Four findings change the model:

- **`meta wait` blocks nothing.** `Ledger::waiting` is read only by status, the idle alarm and the timeline. `meta claim`, `meta review` and `meta merge` never look at it. A wait pauses deadlines and hides the task from the idle alarm, so today it only makes a task quieter.
- **CI cannot read the log.** The log is a local file. A check of "a matching owner event in the log" cannot run on GitHub. For anything CI enforces, the owner's answer must be a GitHub fact.
- **GitHub cannot tell the owner from an agent.** Agents use the owner's own admin token. `Cinglein` is the repo's only collaborator, and it authored and merged the last three PRs with no GitHub reviews. Agents could remove any rule set on this repo, and a required review would stop every PR, because GitHub refuses an author's approval of their own PR. This is open question 1, and most of "CI can check" depends on it.
- **Unclassified must not mean blocking.** The rule "defaults to the owner" would flood the owner if it also stopped work. Who decides and what stops are separate fields here.

## Types

Proposed, and shown together here; one struct per file when built. `OwnerQuestion`, `OwnerAnswer`, `Reason`, `TaskId` and `LeaseId` exist. The rest are new.

```rust
/// One step of one task. Steps are the ones the idle alarm already reasons about.
struct Target { task: TaskId, step: Step }
enum Step { Start, Review, Merge }

/// At least one element by construction.
struct NonEmpty<T> { first: T, rest: Vec<T> }

/// There is no empty form, and no `All`. A repeated target does not parse.
struct Selector(NonEmpty<Target>);

/// The recommendation is one of the options, by construction.
enum Options {
    Stated { recommended: Choice, others: NonEmpty<Choice> },
    /// Only the fold of an old `WaitingOnOwner` event builds this.
    Unstated,
}

enum Class { Listed(ClassId), Unclassified }

/// What the item does to work until it is answered.
enum Hold {
    Blocks { class: ClassId, selector: Selector },
    Proceeds { class: Class, meanwhile: Meanwhile },
    Deferred { class: Class, until: Trigger },
}
enum Trigger { TaskEnds(TaskId), OwnerRaises }

struct Decision<Stage> { id: DecisionId, question: OwnerQuestion, options: Options, hold: Hold, stage: Stage }
struct Asked { since: Timestamp, leases: BTreeSet<LeaseId> }
struct Answered { answer: OwnerAnswer, source: Source }
struct Withdrawn { reason: Reason }
```

What these forbid:

| Invalid state | Why it cannot be written |
|---|---|
| Blocking with no selector | `Blocks` holds a `Selector`, and a `Selector` holds at least one target. A log line without one does not parse, and the log already fails closed on that |
| Blocking "everything" | No variant means all. A selector names tasks that exist, so it cannot reach a later task |
| Blocking with no class | `Blocks` takes a `ClassId`, not a `Class`. `Unclassified` can only proceed or defer |
| No recommendation, or one that is not an option | `Options::Stated`. `meta owner ask` has no way to build `Unstated` |
| "Proceeds" with silence read as consent | `meanwhile` must say what stands until the answer. It is often the stricter reading, not the recommendation |
| An answer with no source | `Answered` takes a `Source`, the `owner YYYY-MM-DD: <words>` form that `self/rules.toml` uses |

**Whole-project hold.** It is not a `Hold` variant and not a meta event, so no agent can write it to the log. It is GitHub's `lock_branch` on main, set by the owner by hand. GitHub refuses the merge and `meta merge` shows why. Claims go on, because work on a branch harms nothing. It is owner-only once question 1 is settled.

**Classes are data.** A proposed `self/decisions.toml` has one entry per class. An entry with no source does not parse. The test of a class: do the options trade off something the owner owns? Sources marked "to confirm" are the lead's notes of owner feedback, not the owner's words, and the owner confirms or replaces them (question 6).

| Class | Decider | Source |
|---|---|---|
| `gate` | owner | owner 2026-10-27: budgets and gates are owner-only |
| `outward` | owner | to confirm: rulesets, settings, installs, irreversible actions |
| `architecture` | owner | to confirm: architecture direction |
| `rule-meaning` | owner | prereg, "Fail fast": a falsified prediction means an owner discussion. To confirm for other rules |
| `fix-design` | lead | owner 2026-10-27: "I don't see why it's blocking on me" |
| `triage` | lead | prereg, "Agents": the project agent triages findings; the lead has the final call on architecture-level ones |
| `task-design` | agent | prereg, "Agents": the project agent splits tasks and owns the project design |
| `legacy-wait` | owner | `docs/stage1.md`: the class of an old `WaitingOnOwner` event; `meta owner ask` refuses it |

`meta owner ask` refuses a class whose decider is not the owner, and says "yours to decide". Lead and agent decisions therefore never reach the owner's lists.

## How it replaces `meta wait`

`meta wait` and `meta unwait` go. Each of their effects moves to the selector, so there is one mechanism.

| Today | With decisions |
|---|---|
| `meta wait <task> --owner <q>` | `meta owner ask <q> --class <c> --recommend <a> --or <b> --blocks <task>:start`, with `--blocks` repeated as needed |
| `meta unwait <task> <answer>` | `meta owner answer <decision> <answer> --source <source>` |
| The whole task, always | The named steps. Waiting's old meaning is the selector with all three steps |
| Every lease of the task pauses | A lease pauses only if its own step is held: work lease on `Start`, review lease on `Review`. A hold on `Merge` pauses nothing |
| The task leaves the idle alarm | Only if its next step is held |
| Nothing is refused | `meta claim`, `meta review`, `meta worker start` and `meta merge` refuse a held step and name the decision |
| `Waiting` in `waiting.rs` | Gone; its covered leases move to `Asked` |

New events are `Asked`, `Decided` and `Withdrawn`. Old events stay in the log and fold into decisions: `WaitingOnOwner` becomes an item with `Options::Unstated`, class `legacy-wait` and all three steps of its task, and `OwnerAnswered { task, answer }` settles it. Replayed deadlines do not change. Each of the four open waits (task-21, 36, 37, 52) is withdrawn, then filed in the new form or handled by its class. Task-21, a discussion for later, moves to `meta defer`.

When a task ends, its targets fall away. An item with none left moves to immediate as "holds nothing now". `meta owner withdraw <decision> <reason>` drops an item.

## Reading the selector

The dispatcher is `meta next`, a pure read of the ledger. The lead reads it now. The supervisor from task-36 will call the same function, up to `concurrency.running`.

```
for each active task, not a critic task:
    step = next_step(task)                 # no step: a live lease or a live review has it
    held = Asked items whose Blocks selector holds (task, step)
    held empty: startable, in task order
    else:       skipped, naming each decision
```

`next_step` comes from the cases `stalled.rs` already tells apart: open, claimed or failed review is `Start`; linked with no verdict on its head is `Review`; pass on head and not merged is `Merge`. The dispatcher and the idle alarm share it, so they cannot disagree.

| Command | Given | Result |
|---|---|---|
| `meta next` | a task's next step is not held | listed as startable |
| `meta next` | it is held | listed as skipped with the decision ids; later tasks are still read |
| `meta claim`, `review`, `worker start`, `merge` | the step is held | refused, naming the decision; nothing recorded |
| `meta owner ask --blocks` | the item would alone hold every startable step, and more than one task has one | refused: this is a whole-project hold, and only the owner sets that |
| `meta owner ask --blocks` | the class's decider is not the owner, or a target's task is unknown or not active | refused |
| `meta owner ask --blocks` | a target is `Merge` on a linked task | also turns its PR to draft, which GitHub will not merge; if `gh` fails, refused and nothing recorded. `meta link` does the same for a task linked later |
| `meta owner answer` | the item is asked | event `Decided`; covered leases get the wait back; a draft PR with no other hold is marked ready |
| idle alarm | no running work, and a next step is not held | fires as today |
| idle alarm | no running work, and every next step is held | fires: "every next step is held", naming the decisions. A wait no longer silences it |

An agent whose own step is held ends its turn. That stops one lease, and the next `meta next` starts other work. So a blocking item is a poor excuse to stop: it names one step, and the alarm fires if nothing else runs.

## Status views

No list is stored. All three are filters on `Hold` over `Decision<Asked>`:

| View | Filter | Silence means |
|---|---|---|
| blocking | `Blocks` | Those steps stay held; all others go on |
| immediate | `Proceeds` | `meanwhile` stands; no timeout turns it into consent |
| deferred | `Deferred` | Comes back to immediate when its trigger fires |

`meta owner` prints the three in that order. `meta status` prints the blocking view in place of "waiting on the owner (flagged)", and counts of the other two. `meta defer` on a task stays as it is, since a deferred task is not a question; its section stays beside this view.

```
blocking
  decision-1 [outward] launchd job for the supervisor, or another way?
    recommend: launchd user agent; or: start by hand each session
    holds: task-36 start, task-37 start (236 min)
    answer unblocks: task-37 start; task-36 start stays held by decision-2
```

"Answer unblocks" is computed, not written: the targets whose steps have no other asked item, and which are each task's next step. A target held by two items says so.

## What CI can check

CI means the required `gates` job and GitHub's own rules. Local refusals in the section above are scheduling, not gates.

| Check | Where | Needs |
|---|---|---|
| A change to gate paths needs the owner: code-owner review on `.github/`, `ci/`, `self/`, `clippy.toml`, the lint tables and `docs/prereg.md`, stale approvals dismissed. This closes #14, including the workflow-from-the-PR's-branch hole, because a PR that edits its own gates cannot pass without the owner. The review is the owner's answer; meta records it when it reads the PR | GitHub branch protection | Question 1 |
| The code-owner paths equal the `gate` class's paths in `self/decisions.toml`, so the two cannot drift | test in `gates` | Nothing |
| `self/decisions.toml` parses: each class has a decider and a source | test in `gates` | Nothing |
| A selector with no target, and an `Unclassified` block, do not compile or parse; behaviour tests list each failure | test in `gates` | Nothing |
| A PR held on `Merge` cannot merge while it is a draft | GitHub | Question 1, or an agent can mark it ready |
| A whole-project hold stops merges | GitHub `lock_branch` | Question 1 |

GitHub behaviour here comes from its documented rules and has not been probed. The first implementation step is the test #14 names: a PR that touches `ci/` and is blocked.

What CI cannot check:

- **A log entry's truth.** `meta owner answer` is typed by the lead. `Source` proves a format, not that the owner said it. The critic can compare it with the record; nothing mechanical can.
- **`Start` and `Review` holds.** These are local. An agent that ignores meta can do the work anyway, and it only meets a gate at merge.
- **Whether a class was chosen honestly.** A gate change labelled `fix-design` is still stopped by path. An outward action is not: installing launchd, or editing live hook settings by hand as on Oct 31, touches no file in a PR.
- **Whether a recommendation is good, or a selector is as narrow as it could be.** Review and the critic read these.
- **Who filed an item.** Meta has no caller identity (question 3).

## Worked table

The eight open items, as they would be filed. Selectors use tasks that exist today.

| Item | Class, decider | List | Holds | Keeps going | Recommendation, other option | Answer unblocks |
|---|---|---|---|---|---|---|
| launchd supervisor | `outward`, owner | blocking | task-36 start, task-37 start | All else | launchd user agent; or start by hand | task-37; task-36 once the next row is answered too |
| Alert or stop on a falsified prediction (#48; P1, P2 read FALSIFIED) | `rule-meaning`, owner | blocking | task-36 start: it would add a route for starting sessions, which is the hole P1 points at | All else | Alert, plus a blocking item on the work the failed prediction names; or stop all, which is `lock_branch` | Together with the row above, task-36 |
| #53 (task-52) | `fix-design`, lead | none | Nothing; the wait is withdrawn | Fix and re-review | The fail-closed tree check keeps every promise, so the lead decides. Merging over a failed review would be `gate` and hold `task-52 merge` only. The route question in #54 is separate | Nothing to answer |
| Worker-path canary (#55, Q2) | `rule-meaning`, owner | immediate | Nothing | Meanwhile: each canary is recorded first as `outside-start` and counts as a bypass | An allowed step with its own record; or it stays a bypass | Nothing held; P2 stops counting canaries |
| #14 gate changes | `gate`, owner | immediate | Nothing | Meanwhile: `review-gates` flags loosening in review | Code-owner review with a separate agent identity; or flag only | The first check in the CI table |
| #12 agent starts outside the hook | `rule-meaning`, owner | immediate | Nothing | Meanwhile: such a session counts as a bypass | It counts, and the route closes for agents, with the canary row deciding the one exception; or it stays open and recorded | Nothing held; the route's task can be opened |
| Docker images | `Unclassified`, owner by default | deferred until `OwnerRaises` | Nothing | Meanwhile: "No Docker" stands in briefs | Keep "No Docker"; or state the question. I found no record of it beyond that constraint and one thermal note | Nothing |
| Version floor | `gate`, owner | immediate | Nothing | Meanwhile: the floor stays 2.1.314 | Raise it to at least 2.1.316, with the fix #47 cites, after a new flag probe; or keep it | Nothing held. If the owner wants the floor raised before workers read more, add `task-52 merge` |

Effect on today's board: `meta next` skips task-36 and task-37. Task-52's fix and tasks 56, 59, 60 and 62 keep moving. The owner's blocking list has two items, and both hold only the supervisor's tasks. Task-52's wait goes, and task-21's becomes a deferral.

## Open questions for the owner

1. **Do agents get their own GitHub identity, write and not admin, while you keep `Cinglein` as admin?** Recommend yes. Without it, "gh should enforce that" has nothing to enforce against, since the agent's token can undo any rule. It needs a second account, and it may change rule `identity`.
2. **Is the gate path list above right, with every PR on it waiting for you?** Recommend yes, with no exemption for tightening, which a path check cannot see. The cost is one held merge per such PR; work on it goes on. This is the largest new demand on you.
3. **Who may file `Blocks`?** Recommend the lead only; agents file `Proceeds`, and the lead promotes. Meta has no caller identity, so this is a brief rule until each leased agent's calls can be told apart.
4. **Is `lock_branch` the whole-project hold, and is it the "stop" in prereg's "Fail fast"?** Recommend yes for the first. For the second, recommend narrowing "Fail fast" to the worked table's row, so that a falsified prediction holds what it points at. P1 and P2 read FALSIFIED today, so reading it literally means stopping now.
5. **Should an unanswered item ever time out?** Recommend no. Time never turns silence into an answer (#48). The alarm covers stalls, because other work keeps starting.
6. **Are the classes above right, and do the "to confirm" sources stand?** Recommend adopting them as `self/decisions.toml`. The #53 row is the test: under these classes, that wait would not have reached you.
7. **What is the Docker images question?** Recommend you or the lead restate it. It is filed only because the rule gives anything unclassifiable to you.

## Building it

- Implementation touches `self/` and needs an owner decision on `self/decisions.toml`, as this spec cannot make one. It does not change `docs/prereg.md`; question 4 may.
- It adds a check, so `red-first` applies. `meta next` reads the log, so `live-reads` applies.
- Order: identity and code-owner review first, because it is settings, closes #14, and needs no meta code. Then types and `meta owner`. Then refusals and `meta next`. Then retire `meta wait`.
