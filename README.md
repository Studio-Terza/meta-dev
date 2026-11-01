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
It is report only, no number blocks. The run also lists public `is_` and `has_` methods returning `bool`
(`bool_methods`), with the headline naming the alternatives; that list is report only too, so it is outside
the lints above and never changes the exit code.

Every lint implements `Lint` and takes the same files. Limits are constants inside each lint, not
settings, and there is no per-repository configuration, switch or baseline. Where something is sometimes
legitimate, the lint gates where by naming convention: starting a process (`process_starters`) is allowed only
in a crate whose name ends in `ports` or a module named `ports`, and casts (`const_shape`), bare numbers
(`named_numbers`) and bare primitives (`primitive_boundary`) only in a module named `bits`. Files under
`tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

Crates: `lint-core` (the `Lint` trait, reports, sites, violations, source files), `lint-layout` (names, matching, manifests, failures), `lint-placement` (where a file sits: package, crate root, module path), `lint-roots` (package and workspace discovery), `lint-reader` (reads a tree into source files), one `lint-<name>` crate
per lint (`lint-crate-graph` among them), `lint-item-graph` (the item graph `crate_graph` reads), `lint-fixtures` (shared test sources), `lint-runner` (the runner
library), `lint` (the binary), `lint-bool-methods` (the report-only list), `golden` (golden files for tests), `laws` (law macros for tests), `newtype` (the newtype traits) and `newtype-derive` (their derives). Crates
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

## Newtypes

`#[derive(Newtype)]` implements `Newtype` (`type Inner`, `new`, `into_inner`) for a struct with one field,
tuple or named, generic or not.

    #[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Rounds(usize);

A validated newtype has no `new`. With `#[newtype(check = ..., error = ...)]` the same derive implements
`Validated` instead, and `try_new` is the only way in. The check is a function from `&Inner` to
`Result<(), Error>`.

    #[derive(Newtype, Arbitrary, Debug)]
    #[newtype(check = Even::check, error = Odd)]
    struct Even(u32);

`#[derive(Arbitrary)]` implements proptest's `Arbitrary` through those constructors, so a validated newtype
only generates values its check admits. The using crate needs `proptest` as a dependency.

`Counter` is a blanket trait over every `Copy` newtype of `usize`: `zero`, `count`, `plus`, `incremented`
and `total`, saturating at the largest count. The orphan rule forbids a blanket `Add` or `Sum`, so these
are named methods. Derived code is not in the source, so `repeated_methods` does not see it, and a
hand-written constructor on three types still fails.

## Mutants

The `mutants` job in `ci.yml` runs on each pull request. It applies `cargo mutants` to the lines the pull
request changes, and tests each mutant with the whole workspace, because shared crates are tested through
the crates that use them (`.cargo/mutants.toml`). A mutant that no test catches fails the job. A mutant that makes the tests hang counts as caught, because no test can show more.

`mutants.yml` runs every mutant once a week, in eight shards, or on request. It reports in the run
summary and the shard artifacts. Missed mutants do not fail it; a shard fails only if the run itself
fails. No push starts it and no push cancels it.
