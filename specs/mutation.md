# mutation

Problem: know which code no test covers and which tests prove nothing, without slowing every PR.

L1 PRs run only mutants in changed lines.
L2 One occasional full run on main, not on every push, and a push never cancels it.
L3 The full run reports and blocks nothing.
D1 PR gate: cargo mutants with --in-diff, in CI.
D2 Full run on a schedule, sharded, with no cancel-in-progress and shard limits long enough to finish.
D3 Full run records which tests kill which mutants: surviving mutants, tests that kill nothing, tests that fail where goldens do not. after: D2
D4 Incremental reruns (owner d2edfb63): a push to a PR mutates against the newest of its last ten completed ci runs that ran on an ancestor of the new head and left every shard's result (an artifact of the workflow, never local state); it mutates only that run's MISSED survivors, mapped to their current lines, and the PR-diff mutants on lines changed since the tree that run tested, plus every source line of a crate whose tests the PR first changed since then (a crate whose tests the earlier run's diff already changed was already mutated whole there, so adding to its tests selects only survivors and changed lines). A first run, a run with no complete prior on an ancestor (a force-push, a cancelled or partial run, expired artifacts), and a run whose own commits since the prior change test code in any way but pure additions fall back to a full run; test code is files under tests/, files holding #[cfg(test)], and crates that are only ever dev-dependencies, read from cargo metadata. A merge from main stays incremental: main's changes can un-exercise a PR mutant the prior run caught, and that gap is left to the widened sweep on main after merge (owner's "c" ruling). The merge gate is unchanged: every survivor must be killed and every changed line is mutated. Run cancelling stays as it is. after: D1
D5 Hangs end: a PR or sweep mutation shard job stops at 45 minutes, about 1.5 times the slowest seen (31; owner 1092c4d5), and every cargo-mutants run carries --build-timeout 600, so a mutant that hangs its build ends as TIMEOUT, excused like a test that hangs (D1), instead of holding the shard until the job dies. A mutated build script can loop: in #622 Answered::next returning Some forever made design-messages' build script print without end and hung shard 8 for four hours. after: D1
Out: local mutation runs, thresholds, repo1's mutation tooling beyond what is needed.
