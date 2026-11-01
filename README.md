# meta-dev

Source lints for Rust repositories, run through one binary.

    cargo run -p lint -- <path>

The runner finds the packages under the path (a directory with a `Cargo.toml` holding `[package]`,
listed by its workspace when one encloses it; hidden directories and `target` are skipped), reads each
package's crate roots and the modules they declare, runs all lints, prints the violations, and exits
non-zero if there are any. Whether a file is shipped code, a binary or a test is decided from its place
inside its package, so the verdict does not depend on where the checkout lives.

Lints: no_comments, no_free_fns, thin_main, vec_out_params, named_lifetimes, enum_predicates,
const_shape, type_shape, distinct_signatures, expression_flow, fn_shape, literal_names,
manual_iteration, module_nesting, named_numbers, no_forwarders, no_numbers_in_binaries,
no_parameter_bags, primitive_boundary, private_fns, process_starters, public_surface,
repeated_methods, state_graph, crate_graph, public_api_tests, no_test_only_api, no_ignored_tests,
near_duplicate_tests.

Every run also prints the test budget: tests, files, lines, test lines and process starts per package, counting law macros as tests. Process starts
are counted everywhere, behind `ports` or not; only those outside `ports` are violations.
It is report only, no number blocks.

Every lint implements `Lint` and takes the same files. Limits are constants inside each lint, not
settings, and there is no per-repository configuration, switch or baseline. Where something is sometimes
legitimate, the lint gates where by naming convention: starting a process (`process_starters`) is allowed only
in a crate whose name ends in `ports` or a module named `ports`, and casts (`const_shape`), bare numbers
(`named_numbers`) and bare primitives (`primitive_boundary`) only in a module named `bits`. Files under
`tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

Crates: `lint-core` (the `Lint` trait, reports, sites, violations, source files), `lint-layout` (names, matching, manifests, failures), `lint-placement` (where a file sits: package, crate root, module path), `lint-roots` (package and workspace discovery), `lint-reader` (reads a tree into source files), one `lint-<name>` crate
per lint (`lint-crate-graph` among them), `lint-item-graph` (the item graph `crate_graph` reads), `lint-fixtures` (shared test sources), `lint-runner` (the runner
library), `lint` (the binary), `golden` (golden files for tests) and `laws` (law macros for tests). Crates
stay split by lint because `crate_graph` flags a crate whose items fall into two loosely linked groups of
five or more, or whose cycle spans more than 40 items; its conductance floor is 19%.

## Goldens

A golden is a recorded output of a public API, kept at `tests/goldens/<name>` in the package.

    #[test]
    fn report_is_stable() -> Result<(), golden::Failure> {
        golden::golden!("report.txt", &render_report())
    }

`cargo test` only compares. It fails on a difference or a missing golden, and it writes nothing. The
failure names the file and shows a diff in which each line is quoted, so a change in whitespace or line
endings is visible. The diff is one hunk, from the first changed line to the last.

To change a golden, run `GOLDEN_RECORD=1 cargo test` and review the diff. Any other value of
`GOLDEN_RECORD` fails, and so does recording when `CI` is set.

## Laws

A law states a property in one line and generates one `#[test]`. The strategy is a proptest strategy,
`laws::proptest` is the re-export, and functions take their input by reference.

    laws::roundtrip!(hex_recovers_the_number, any::<u32>(), Hex::render, Hex::parse, Ok);
    laws::idempotent!(sorting_twice_changes_nothing, vec(any::<u8>(), 0..32), Numbers::sorted);
    laws::table!(hex_examples, Hex::parse, ["ff" => Ok(255), "0" => Ok(0)]);

`roundtrip!` checks `decode(encode(value)) == wrap(value)`; `wrap` is optional, and `Ok` or `Some` suits
a fallible decode. `idempotent!` checks `f(f(value)) == f(value)`. `table!` checks each `input => want`
row and names the first broken row, without generated input. Attributes before the name go on the test.

Each run uses one fixed seed and writes no regression files, so a run is repeatable and leaves the tree
alone. A failure prints the smallest failing input. `PROPTEST_RNG_SEED` explores another seed.

## CI wall-time limits

`ci/limits.toml` holds a warn and a block level in seconds for each timed CI stage: toolchain, clippy,
test compile, test run and lint. `ci/timed.sh` times a stage; the test run is the sum of the `finished in`
times libtest prints, so compile time is not in it. The `limits` step runs `gate-limits`, which prints
each stage as ok, WARN or BLOCK and fails when any stage is above its block level. A stage with a limit
but no measurement, or a measurement with no limit, also fails. The levels come from seven cold runs
on `ubuntu-latest`: block just above the slowest run, warn just above the median.
No cache is used: caching was measured and did not pay for its save time.
