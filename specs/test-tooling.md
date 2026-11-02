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
D5 Mandatory laws from code shape: a type with both `Display` and `FromStr`, or `Serialize` and `Deserialize`, or a pair like encode/decode or to_x/from_x, needs a `roundtrip!`. Hand-written `Ord` or `PartialEq` needs a law. A lint enforces it, in its own crate, added to the `lint` runner. after: D2
D6 A golden change passes CI only inside the PR's reach. The reach is what `reach` computes for segmentation from every file the PR changes except the goldens and new scenario files themselves: the crates those files belong to and every crate depending on them. A changed golden passes without a label when every crate owning it is in that reach. A golden of a crate outside the reach fails: nothing the PR changed can produce it, so it is a hand edit or leakage. A PR whose title starts with `refactor:` fails on any golden change, since a refactor leaves behaviour unchanged. The label `golden-change` is the owner's escape for a case the rule refuses; agents never add it. after: D1
D7 New files under tests/fixtures or tests/e2e follow the reach rule of D6, with the label `new-scenario` as the owner's escape. The `refactor:` title does not apply to them. after: D1
Out: cargo mutants, fuzzing, e2e tiers, repo1's test helpers.

## History

D6, D7: label approval replaced by the reach rule, owner's message 162b206d.
