# Stage 0: the minimal enforced loop

Stage 0 is the smallest `meta` that runs the lead's real loop end to end. Once it lands, the lead cannot start an agent or merge a PR except through it. Bootstrap order and the hypothesis being tested: `docs/prereg.md`.

## Behaviours

Each row below is one behaviour-table test. A test drives `meta` through a single in-process `Call` against an in-memory log and recorded replies from the git and GitHub ports.

| Command | Given | Result |
|---|---|---|
| `meta open <title>` | any | prints a new task id; event `Opened` |
| `meta claim <task> --agent <name>` | the task is open and unclaimed | event `Claimed` plus `Leased` with a fresh lease id; prints the brief: the task, the setup steps, and every rule in `self/rules.toml` with its source |
| `meta claim` | the task is already claimed, done or unknown | refused, with the reason |
| `meta link <task> <pr>` | the task is claimed | event `Linked` |
| `meta merge <pr>` | the PR is linked; required checks are green on its current head; the head equals the pinned SHA | squash-merges through the GitHub port with that head pinned; events `Merged` and `Done` |
| `meta merge <pr>` | the PR is unlinked, has a red or missing check, or its head moved | refused; names which, and gives the next step |
| `meta escape <reason>` | any | event `Escaped`; prints the open escape count and how many times this reason has repeated |
| `meta status` | any | open tasks, live leases and open escapes, plus the counters in "Measures" |
| `meta hook pre-tool-use` with tool `Agent` | the prompt carries a lease id that is live and unused | allowed; event `LeaseUsed` (each lease is single use) |
| `meta hook pre-tool-use` with tool `Agent` | no lease, an unknown lease, or an already used lease | denied, with how to get one (`meta claim <task> --agent <name>`) |
| `meta hook pre-tool-use` with tool `Bash` | the command merges a PR or pushes to `main` any way other than `meta merge`: `gh pr merge`, `gh api …/merge`, `git push … main` | denied, pointing to `meta merge` or `meta escape` |
| `meta hook pre-tool-use` | input that cannot be parsed | denied (fail closed) |

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
