# segmentation

Problem: a change should only pay for what it can affect, and how far a change reaches is the measure of tangle.

L1 PRs check only crates a change can reach, from the Cargo dependency graph.
L2 A change to workspace-wide files reaches everything.
L3 Main always runs everything.
D1 Reach tool: changed files to affected crates, no process spawning in tests.
D2 PR CI runs clippy, tests, lints and mutants on the reach only. after: D1
D3 Report reach per PR, and a per-crate fan-in, on main. after: D1
D4 Prove it on a real PR: an unrelated change runs a fraction of the suite. after: D2
Out: elegance lints, test-level selection, runner noise.
