# lint-port

Problem: generalize the existing lints so any repository can adopt them.

L1 Lints run through one entry point and depend on no orchestration code.
L2 This repository passes its own lints in CI, with no allow-lists.
L3 Tests go through public APIs and spawn no processes.
L4 No per-repository configuration. Lints gate where by naming convention.
D1 First slice of small lints ported.
D2 Remaining lints and tooling ported. after: D1
D3 Chess moves code to the conventions, fixes its violations and runs the ported lints in CI, replacing meta-xtask 0.8.0. after: D2
Out: elegance lint, cargo mutants, test budgets, orchestration crates.
