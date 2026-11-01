# segmentation

Problem: a change should only pay for what it can affect, and how far a change reaches is the measure of tangle.

L1 PRs check only crates a change can reach, from the Cargo dependency graph.
L2 A change to workspace-wide files reaches everything.
L3 Main, and every PR into main, runs everything.
D1 Reach tool: changed files to affected crates, no process spawning in tests.
D2 PR CI runs clippy, tests and mutants on the reach only, and lints on the whole repository, since crate_graph cannot judge a slice. after: D1
D3 Report reach per PR, and a per-crate fan-in, on main. after: D1
D4 Prove it on a real PR: an unrelated change runs a fraction of the suite. after: D2
D5 A PR reaching a fraction of the crates takes a matching fraction of job time. after: D4
D6 PRs into main run the full suite and the stage limits; PRs into project branches run only their reach. after: D2
Out: elegance lints, test-level selection, runner noise.
