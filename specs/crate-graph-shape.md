# crate-graph-shape

Problem: the crate dependency graph can turn into spaghetti, with logic sitting in crates that most of the workspace depends on, so one edit rebuilds and retests nearly everything. This is the first of two compile-time concerns, judged only by the shape of the graph as it stands.

L1 Everything here reports and blocks nothing, until the owner decides otherwise from the data.
L2 A snapshot only: the inputs are the workspace as it stands. No git history, churn, volatility or timing anywhere, so a refactor is free and only its result is judged.
L3 Syntax and manifests only, no per-repository configuration, no allow-lists.
L4 One model: the crate graph is `reach`'s, read from `cargo metadata`; logic is counted by `lint-complexity`; findings print through `lint-core` reports. No parallel graph, complexity count or report path.
L5 Shipped code only: files under `tests`, `examples` and `benches`, and code under `#[cfg(test)]`, are not counted.
D1 Report: reach per crate, the number and share of workspace crates that depend on it, directly or transitively, and their distribution: crates per tenth of the share, the median and the largest. `reach fan-in` prints it; on main, CI writes it to the job summary.
D2 Report: hub thickness per crate, the logic a crate holds, its shipped functions' cyclomatic complexity above one summed, and that logic weighted by D1's dependent count. A crate with high fan-in should be thin, types, traits and interfaces, with the logic in the leaves. Crates are listed heaviest weighted first, with the share of workspace logic held by crates that more than half the workspace depends on. On main, CI writes it to the job summary. after: D1
D3 D1 and D2 are scored against the repo1 smell inventory as elegance#D8 and transition-graph#D5 were: recall against a random control of the same size, act rate in a sample, determinism, gate material, outside this repository. after: D2
D4 Gate candidates go to the owner with their numbers; nothing blocks CI. after: D3
Out: gates, git history, churn and volatility, compile timings (the second concern), public-API change detection, the item graph inside one crate (`crate_graph`).

## History

Owner's messages 42dad9d0, 772f1172, 871be1b4.
