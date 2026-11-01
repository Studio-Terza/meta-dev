# mutation

Problem: know which code no test covers and which tests prove nothing, without slowing every PR.

L1 PRs run only mutants in changed lines.
L2 One occasional full run on main, not on every push, and a push never cancels it.
L3 The full run reports and blocks nothing.
D1 PR gate: cargo mutants with --in-diff, in CI.
D2 Full run on a schedule, sharded, with no cancel-in-progress and shard limits long enough to finish.
D3 Full run records which tests kill which mutants: surviving mutants, tests that kill nothing, tests that fail where goldens do not. after: D2
Out: local mutation runs, thresholds, repo1's mutation tooling beyond what is needed.
