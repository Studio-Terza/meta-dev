# transition-graph

Problem: the same behaviour written with different control flow passes a body-shape report. Effects spread through the code, and signatures over bare primitives, hide what a function does.

L1 Everything here reports and blocks nothing, until the owner decides otherwise from the data.
L2 Syntax only, no per-repository configuration, no allow-lists.
L3 One model: the graph is read from `type-facts`, as `lint-type-graph` is, in its own crate since `crate_graph` splits unrelated graphs; paths resolve through `lint-paths`, crates come from `lint-placement` and `reach`, and findings go through `lint-core` reports and `lint --report`. No parallel graph or report path.
L4 Shipped code only: files under `tests`, `examples` and `benches`, and code under `#[cfg(test)]`, are not read.
D1 Report: effect sites outside ports, per crate. An effect is a call into files, environment, clock, network, processes, randomness or stdio. Ports are what `process_starters` and `raw_resources` already allow: a crate whose name ends in the word `ports`, or a module named `ports`. Sites are counted everywhere, in ports or not, by kind, as process starts are in the test budget; only those outside ports are findings.
D2 Report: the transition graph of shipped code. Nodes are types; edges are functions, methods and trait impls, from each input type, the receiver included, to the output type, read through references, `Option`, `Result` and collections to the type inside. A type name resolves to the crate's own type first, else to the one workspace crate declaring it; a name declared in several other crates, a foreign type or a generic is a leaf. On main, CI writes per-crate nodes, edges and coverage to the job summary. after: D1
D3 Coverage: per crate, the share of edges whose inputs and output are all domain types rather than bare primitives (`bool`, integers, floats, `char`, `String`, `&str`, unit). A primitive edge is invisible to the graph. Coverage is printed with D2, as the number the `primitive_boundary` and `bool_methods` lists should raise. after: D2
D4 Report: duplicate transitions, a pair of types joined by more than one edge in shipped code, naming the pair and each path. A standard conversion trait (`From`, `TryFrom`, `FromStr`) is the canonical edge, and the message says to route the others through it. Edges touching a primitive are counted apart, since many legitimate functions share `Foo` to `bool`. after: D2
D5 D1 and D4 are scored against the repo1 smell inventory as elegance#D8 was: recall, act rate in a sample, determinism, gate material, outside this repository. after: D1, D4
D6 Gate candidates go to the owner with their numbers. after: D5
Out: gates, moving effects into ports, cyclomatic complexity (elegance#D4), `states!` lifecycle graphs (`state_graph`).

## History

Owner's messages 871be1b4, ccce1a26, eed62ccb; newtypes as distinct nodes, 673813ba. Bare primitives as edges the graph cannot capture: owner's message of 2026-11-02, not yet indexed.
