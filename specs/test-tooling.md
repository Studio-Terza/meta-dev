# test-tooling

Problem: agents write tests that bloat the suite and pin implementation. Writing a good test should be the easy path.

L1 Goldens check black-box output, run with plain cargo test, and spawn no processes.
L2 A golden changes only when someone deliberately re-records it. A test run never rewrites one.
L3 Writing a law is shorter than writing an example test.
L4 Tests go through public APIs.
D1 Golden crate: compare, record through an explicit command or environment variable, clear diff on failure.
D2 Law macros: roundtrip, idempotent, table, built on proptest. after: D1
D3 Test design lints: public API only, no test-only API, no ignore, near-duplicate tests. Added once lint crates are stable.
D4 Test budget per crate. after: D3
Out: cargo mutants, fuzzing, e2e tiers, repo1's test helpers.
