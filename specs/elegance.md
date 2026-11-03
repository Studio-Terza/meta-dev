# elegance

Problem: agents add copies instead of generalizing, and we lack data on how much.

L1 Everything here reports and blocks nothing, until the owner decides otherwise from the data.
L2 Syntax only, no per-repository configuration.
D1 Report: repeated shapes (`repeated_shapes`): a function or method body, closure, block or match expression of 40 tokens or more that repeats with its names and literals erased, grouped across the workspace over all code (shipped, tests, examples), a group nested inside a larger group dropped, each group named by where its copies live: tests only, shipped only, or both. The fix follows the place: for tests, one table-driven test or a law; for shipped code, extract it once, a generic over an existing trait, or a derive; for both, the test re-implements shipped logic and should test through it. It becomes a blocking lint once its fix pass lands, and then replaces `near_duplicate_tests`.
D2 Blocking lint `unconsumed_traits`: zero local traits with methods that nothing in shipped code bounds on (generic, where clause, supertrait), takes as impl Trait or dyn Trait, or calls through a provided method; the refusal names the fix: delete the trait and keep its methods inherent, or give it a consumer. Promoted by the owner on 2026-11-03; the six findings on main were fixed (Relation, FactsLint and Validated deleted; FileLint gained `in_shipped`, Step a generic parse, GraphLint the generic `OnGraph` lint).
D3 Report: reach per PR over time, as a history file written on main. Fan-in per crate dropped by the owner on 2026-11-03 with crate-graph-shape.
D4 Report: cyclomatic complexity per function, as a distribution per crate. after: D1
D5 Report: type-trait graph, types as nodes, trait impls and conversions (`From`, `AsRef`) as edges, with per-crate nodes, edges, density and largest strongly connected component. Syntax only: edges resolve by name, so a type name shared by two types merges them. after: D1 Dropped by the owner on 2026-11-03; the report and `lint-type-graph` are deleted.
D6 Report: pairs of types with near-identical outgoing method and trait walks, as candidates to unify. Types deriving `Newtype` are excluded, and so are types whose walks diverge; two newtypes over one inner type are distinct nodes on purpose and are never reported for that. after: D5 Dropped by the owner on 2026-11-03 as a duplicate of the equivalence engine (lint-queue D9); the report and `lint-walk-similarity` are deleted.
D7 Report: compile time per crate on main. after: D1
D8 The reports are scored against the smell inventory, hits and misses per report, outside this repository. after: D6
Out: gates, repository-wide scores, repeated-switch checks, widening raw-call bans.

Each finding's message names real fixes in order: a standard trait, a generic over an existing trait, a derive. Never "extract a helper". `lint --report` prints these separately from failures. On main, CI writes them to the job summary.

## History

D1: widened from shipped code to all code and named `repeated_shapes`, with the fix chosen by where the copies live, owner's message 0bcc860a.

D3 fan-in, D5 and D6 dropped (owner faca6011, e698268b): crate reach below chance; type-graph density informs nothing; walk similarity duplicates the equivalence engine.

D2, made blocking (owner e698268b, 6b32f5b9): strict by default, so a provided method counts only when something calls it.
