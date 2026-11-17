# mutation

Problem: know which code no test covers and which tests prove nothing, without slowing every PR.

L1 PRs run only mutants in changed lines.
L2 One occasional full run on main, not on every push, and a push never cancels it.
L3 The full run reports and blocks nothing.
D1 PR gate: cargo mutants with --in-diff, in CI.
D2 Full run on a schedule, sharded, with no cancel-in-progress and shard limits long enough to finish.
D3 Full run records which tests kill which mutants: surviving mutants, tests that kill nothing, tests that fail where goldens do not. after: D2
D5 Hangs end: a PR or sweep mutation shard job stops at 90 minutes (the slowest seen is about 31), and every cargo-mutants run carries --build-timeout 600, so a mutant that hangs its build ends as TIMEOUT, excused like a test that hangs (D1), instead of holding the shard until the job dies. A mutated build script can loop: in #622 Answered::next returning Some forever made design-messages' build script print without end and hung shard 8 for four hours. after: D1
Out: local mutation runs, thresholds, repo1's mutation tooling beyond what is needed.
