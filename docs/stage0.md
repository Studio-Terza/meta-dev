# Stage 0: the minimal enforced loop

Stage 0 is the smallest `meta` that runs the lead's real loop end to end. Once it lands, the lead cannot start an agent or merge a PR except through it. Bootstrap order and the hypothesis being tested: `docs/prereg.md`.

## Behaviours

Each row below is one behaviour-table test. A test drives `meta` through a single in-process `Call` against an in-memory log and recorded replies from the git and GitHub ports.

| Command | Given | Result |
|---|---|---|
| `meta open <title>` | any | prints a new task id; event `Opened` |
| `meta claim <task> --agent <name>` | the task is open and unclaimed | event `Claimed` plus `Leased` with a fresh lease id; prints the brief: the task, the setup steps, and every rule in `self/rules.toml` with its source |
| `meta claim <task> --agent <name>` | the task is claimed or linked, and none of its leases is unused | event `Leased` with a fresh lease id; prints the brief. That is still one lease per agent start |
| `meta claim` | the task is claimed with an unused lease, done, or unknown | refused, with the reason; a claimed task's refusal names the unused lease and says a fresh one is issued once it is spent |
| `meta link <task> <pr>` | the task is claimed; the PR's head branch is the task id; its base is `main`; no other task links the PR | event `Linked`, with the PR's head pinned once GitHub reports the commit last pushed from the local branch `<task>`; polls for up to 60 s |
| `meta link <task> <pr>` | the task is not claimed, the branch or base differs, or another task links the PR | refused; names which, and gives the next step |
| `meta link <task> <pr>` | GitHub still reports an older head than the local branch `<task>`, then reports it within 60 s | event `Linked`, pinned at the pushed commit |
| `meta link <task> <pr>` | GitHub does not report the local branch's commit within 60 s | refused; names both SHAs, and says to push again or reconcile the branches |
| `meta link <task> <pr>` | there is no local branch named after the task | refused; says to commit, push the branch, and link again |
| `meta merge <pr>` | the PR is linked; required checks are green on its current head; the head equals the pinned SHA | squash-merges through the GitHub port with that head pinned; events `Merged` (route `Meta`) and `Done` |
| `meta merge <pr>` | the PR is unlinked, closed, has a red or missing check (the worst report per check wins), has no required checks, or its head moved | refused; names which, and gives the next step |
| `meta merge <pr>` | the PR is behind main (GitHub's merge state is `BEHIND`) | refused; says it is behind main and gives the next step: update the branch, wait for CI on the new head, link the new head, run `meta merge` again; nothing is merged |
| `meta merge <pr> --update` | the PR is linked, open and behind main | the GitHub port updates the branch; no events; prints "updated; CI must pass on the new head before merging"; never merges in the same call |
| `meta merge <pr> --update` | the PR is unlinked, not open, or not behind main | refused; names which; nothing is updated |
| `meta merge <pr>` | the linked PR is already merged at the pinned head | events `Merged` (route `Outside`, not counted as through meta) and `Done`; no second merge |
| `meta merge <pr>` | the protection lookup or the gh replies fail | error, no events |
| `meta escape <reason>` | any, even a log that does not fold | event `Escaped`; prints the open escape count and how many times this reason has repeated |
| `meta status` | any | open tasks, live leases and open escapes, plus the counters in "Measures" |
| `meta status` | every PR merged on GitHub (read through the GitHub port) has a `Merged` event in the log | P1 merges: all through meta (or logged as outside); nothing missing from the log |
| `meta status` | a PR merged on GitHub has no `Merged` event in the log | it counts as "merged outside meta" in P1, and the line says how many are missing from the log |
| `meta status` | the merged-PR list has more than 1000 rows (the adapter asks for 1001, so a full page shows as truncation) | the report still prints; P1 merges says the reconciliation is unavailable because the list is truncated |
| `meta status` | the merged-PR list cannot be read or parsed | the report still prints; P1 merges says the reconciliation is unavailable and why |
| any ledger command | the log does not fold | error naming the log path, then "Next: meta escape <reason>, then repair <path>" |
| `meta hook pre-tool-use` with tool `Agent` or `Task` | the Agent `description` is exactly a lease id that is live and unused (the `prompt` is not read) | allowed; event `LeaseUsed` (each lease is single use) |
| `meta hook pre-tool-use` with tool `Agent` or `Task` | the description is anything but a lease id, or names an unknown lease or an already used lease | denied, with how to get one (`meta claim <task> --agent <name>`) |
| `meta hook pre-tool-use` with tool `Agent` or `Task` | the log cannot be opened, stays locked past its deadline, or does not fold | denied (fail closed), naming the log and the next step |
| `meta hook pre-tool-use` with any other tool, `Bash` included | any `tool_input`, even a merge or a push to `main`; any log state | allowed; the hook parses the input only to find the tool name, does not inspect `tool_input`, and does not read the log |
| `meta hook pre-tool-use` | input that cannot be parsed | denied (fail closed) |
| hook entry in `self/claude-settings.json` | `meta` is missing, crashes or times out | the tool is blocked (`\|\| exit 2`, `onFailure: block`, a timeout longer than the log lock deadline) |

## Measures

`meta status` prints these from the log. They are report-only.

- P1: agent starts and merges through meta, out of all. The hook counts denied agent starts. `meta status` lists the PRs merged on GitHub and compares them with the log's `Merged` events; a merge missing from the log counts as "merged outside meta".
- P2: open escapes, and how often each reason repeats.
- P6: CI runs per merged PR, read from the GitHub port at merge time.

## Enforcement

GitHub branch protection and CI enforce merges and pushes, not anything local. `main` requires a PR, a green `gates` check and an up-to-date branch, and the rule includes admins. The only local hook gates agent starts (`Agent`, `Task`) by lease. `meta status` reports merges that went around `meta` after the fact; it does not stop them.

## Structure

- `meta-core`: events, the fold and every decision. Pure, with no I/O.
- `meta-ports`: traits for log storage, git, GitHub and the clock. Each trait has an in-memory implementation and a real one.
- `meta-cli`: the binary. It parses arguments into a `Call` and answers it.
- Only the real port implementations may start processes or read the environment. A CI check fails if `std::process` or `std::env` appears anywhere else.
- The log is an append-only JSONL file in the lead's local data directory, never in the repo.

## Gates

- **Blocking:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and the process/env check above.
- **Report-only:** test wall time and CPU time.
- No other gate exists until a measured need and an owner decision add one.

## Timebox

Stage 0 has 4 hours of wall time from the first agent start. If it overruns, stop and discuss with the owner. Do not patch around the overrun.
