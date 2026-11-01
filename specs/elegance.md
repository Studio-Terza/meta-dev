# elegance

Problem: agents add copies instead of generalizing, and we lack data on how much.

L1 Everything here reports and blocks nothing, until the owner decides otherwise from the data.
L2 Syntax only, no per-repository configuration.
D1 Report: repeated code bodies in shipped code, with names and literals erased, at a 40-token floor, listing sizes so other floors can be read from the same run.
D2 Report: local traits with methods but no consumer: no bound, impl, dyn or provided method.
D3 Report: reach per PR and fan-in per crate over time, as a history file written on main.
D4 Report: cyclomatic complexity per function, as a distribution per crate. after: D1
D5 Report: type-trait graph, types as nodes, trait impls and conversions (`From`, `AsRef`) as edges, with per-crate nodes, edges, density and largest strongly connected component. Syntax only: edges resolve by name, so a type name shared by two types merges them. after: D1
D6 Report: pairs of types with near-identical outgoing method and trait walks, as candidates to unify. Types deriving `Newtype` are excluded, and so are types whose walks diverge; two newtypes over one inner type are distinct nodes on purpose and are never reported for that. after: D5
D7 Report: compile time per crate on main. after: D1
D8 The reports are scored against the smell inventory, hits and misses per report, outside this repository. after: D6
Out: gates, repository-wide scores, repeated-switch checks, widening raw-call bans.

Each finding's message names real fixes in order: a standard trait, a generic over an existing trait, a derive. Never "extract a helper". `lint --report` prints these separately from failures. On main, CI writes them to the job summary.
