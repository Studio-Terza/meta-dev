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
| `meta link <task> <pr>` | the task is claimed; the PR's head branch is the task id; its base is `main`; no other task links the PR | event `Linked`, with the PR's current head pinned |
| `meta link <task> <pr>` | the task is not claimed, the branch or base differs, or another task links the PR | refused; names which, and gives the next step |
| `meta merge <pr>` | the PR is linked; required checks are green on its current head; the head equals the pinned SHA | squash-merges through the GitHub port with that head pinned; events `Merged` (route `Meta`) and `Done` |
| `meta merge <pr>` | the PR is unlinked, closed, has a red or missing check (the worst report per check wins), has no required checks, or its head moved | refused; names which, and gives the next step |
| `meta merge <pr>` | the linked PR is already merged at the pinned head | events `Merged` (route `Outside`, not counted as through meta) and `Done`; no second merge |
| `meta merge <pr>` | the protection lookup or the gh replies fail | error, no events |
| `meta escape <reason>` | any, even a log that does not fold | event `Escaped`; prints the open escape count and how many times this reason has repeated |
| `meta status` | any | open tasks, live leases and open escapes, plus the counters in "Measures" |
| any ledger command | the log does not fold | error naming the log path, then "Next: meta escape <reason>, then repair <path>" |
| `meta hook pre-tool-use` with tool `Agent` or `Task` | the prompt carries a lease id that is live and unused | allowed; event `LeaseUsed` (each lease is single use) |
| `meta hook pre-tool-use` with tool `Agent` or `Task` | no lease, an unknown lease, or an already used lease | denied, with how to get one (`meta claim <task> --agent <name>`) |
| `meta hook pre-tool-use` with tool `Agent` or `Task` | the log cannot be opened, stays locked past its deadline, or does not fold | denied (fail closed), naming the log and the next step |
| `meta hook pre-tool-use` with tool `Bash` | the command merges a PR or pushes to `main` any way other than `meta merge`: `gh pr merge` (also behind gh flags), a `gh api` call that is not a GET or a graphql mutation, `git push … main` (also via `@`, `--git-dir`, `--work-tree`, `--namespace`, `subtree push`, `--all`, `--mirror`), split by `$(`, backticks or line continuations | denied, pointing to `meta merge` or `meta escape`; event `Denied` |
| `meta hook pre-tool-use` with tool `Bash` | `git push` names no branch other than `main` (bare, remote only, `HEAD`, `@`) | denied: "name the branch: git push origin <branch>" |
| `meta hook pre-tool-use` with tool `Bash` | a `git push` or a REST `gh api` call holds an unexpanded `$`, or a graphql query comes from `$`, `@` or `--input` | denied; spell the value out |
| `meta hook pre-tool-use` with tool `Bash` | the command starts an agent with the `claude` program, for example `claude -p` | denied, with how to get a lease and use the Agent tool; event `Denied` |
| `meta hook pre-tool-use` with tool `Bash` | any other command, including `meta` itself, even when the log cannot be opened | allowed; the log is not touched |
| `meta hook pre-tool-use` with tool `Bash` | a denial whose `Denied` event cannot be written | still denied |
| `meta hook pre-tool-use` with any other tool | any `tool_input` | allowed; the input is not read |
| `meta hook pre-tool-use` | input that cannot be parsed | denied (fail closed) |
| hook entry in `self/claude-settings.json` | `meta` is missing, crashes or times out | the tool is blocked (`\|\| exit 2`, `onFailure: block`, a timeout longer than the log lock deadline) |

## Measures

`meta status` prints these from the log. They are report-only.

- P1: agent starts and merges through meta, out of all. The hook counts denied attempts.
- P2: open escapes, and how often each reason repeats.
- P6: CI runs per merged PR, read from the GitHub port at merge time.

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
