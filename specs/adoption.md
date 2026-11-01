# adoption

Problem: any repository should be able to adopt the lints without reading this repository's code.

L1 One command installs the runner at a pinned version.
L2 No configuration file.
L3 An adopter's CI is a few lines.
D1 Installing: `cargo install --git` at a tag works, and the docs say how. Do not publish to crates.io.
D2 A reusable GitHub Actions workflow or composite action adopters can call. after: D1
D3 One page: conventions (`ports`, `bits`, tests in `tests/`), each lint in a line, how to read a finding. after: D1
D4 A tiny example repository under examples/, proved by running the installed runner in CI. after: D2
Out: orchestration, hooks, per-repository settings, version automation.
