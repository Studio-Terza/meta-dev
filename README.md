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
no_parameter_bags, primitive_boundary, private_fns, process_starters, raw_resources, public_surface,
repeated_methods, state_graph, guard_must_use, crate_graph, public_api_tests, no_test_only_api, no_ignored_tests,
near_duplicate_tests, mandatory_laws.

Every run also prints the test budget: tests, files, lines, test lines and process starts per package, counting law macros as tests. Process starts
are counted everywhere, behind `ports` or not; only those outside `ports` are violations.
It is report only, no number blocks. The run also lists public `is_` and `has_` methods returning `bool`, in impls and in public traits
(`bool_methods`), with the headline naming the alternatives, and public methods that form a pair such as
`start` and `stop` (`paired_apis`), also when the two sit in different impl blocks of one type in a crate, with the headline saying to write a guard. Those lists are report only too,
so they are outside the lints above and never change the exit code.

Every lint implements `Lint` and takes the same files. Limits are constants inside each lint, not
settings, and there is no per-repository configuration, switch or baseline. Where something is sometimes
legitimate, the lint gates where by naming convention: starting a process (`process_starters`) and calling
`fs::remove_dir_all`, `mem::forget` or `env::temp_dir` (`raw_resources`) are allowed only in a crate whose name
ends in the word `ports` (`meta-ports`, not `meta-reports`) or a module named `ports`. Both lints read paths through
`use` items in any order, renames, type aliases, glob imports, re-exports across the files of a crate, qualified
`<T>::f` calls and macro bodies, so `matches!(..., Ok(s) if s.success())` is read like any other call. Inside `ports`,
a raw call must sit in an impl of a type that implements `Drop`: the guard that does the cleanup. In a module named `bits` only: casts (`const_shape`), bare numbers
(`named_numbers`) and bare primitives (`primitive_boundary`). Files under
`tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

Crates: `lint-core` (the `Lint` trait, reports, sites, violations, source files), `lint-layout` (names, matching, manifests, failures), `lint-placement` (where a file sits: package, crate root, module path), `lint-roots` (package and workspace discovery), `lint-reader` (reads a tree into source files), one `lint-<name>` crate
per lint (`lint-crate-graph` among them), `lint-item-graph` (the item graph `crate_graph` reads), `lint-paths` (the paths a crate uses, resolved through its imports, shared by `process_starters` and `raw_resources`), `lint-fixtures` (shared test sources), `lint-runner` (the runner
library), `lint` (the binary), `lint-bool-methods` and `lint-paired-apis` (the report-only lists), `states` (the `states!` macro), `golden` (golden files for tests), `fuzz` (the fuzz targets, outside the default members), `laws` (law macros for tests), `newtype` (the newtype traits) and `newtype-derive` (their derives). Crates
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

A golden name stays inside `tests/goldens`: `golden!("../x")` or an absolute name fails with an error.

To change a golden, run `GOLDEN_RECORD=1 cargo test` and review the diff. Any other value of
`GOLDEN_RECORD` fails, and so does recording when `CI` is set.

The lints have one golden each, in `crates/lint-runner/tests/goldens/lints/<lint>.txt`. It records what the
runner prints for the package in `crates/lint-runner/tests/fixtures/<lint>`, and the test runs the runner from
a temporary copy of that package. The copy turns each `manifest.toml` into a `Cargo.toml`. A fixture has no
`Cargo.toml` of its own, so a run over this repository does not read it.

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

`mandatory_laws` fails a type that has a conversion pair, or a hand-written `Ord` or `PartialEq`, without a law in a
test of the same package that names the type. A pair is `Display` with `FromStr`, `Serialize` with `Deserialize`
(also behind `cfg_attr`), `encode` with `decode`, or `to_x` with `from_x`, in inherent impls or any trait impl, in one
file or several. Each pair needs its own `roundtrip!`: one that names the type and, for method pairs, both methods.
The test title does not count as naming the type.

## Labels

`labels.yml` fails a pull request that changes a golden (including renaming one out of `tests/goldens`) unless it
carries the label `golden-change`, and one that adds, copies or renames a file into `tests/fixtures` or `tests/e2e`
unless it carries `new-scenario`. A person adds the label after reviewing the change. The workflow reads label names
only: it does not check who added a label.

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

`#[newtype(strategy = ...)]` gives the derived `Arbitrary` its own proptest strategy for the inner value, for checks
that admit a small share of it (`strategy = 0u32..1000`, `strategy = "[0-9a-f]{40}"`). A validated newtype with a
strategy still goes through its check. The derive also writes an inherent `const fn new` for a plain newtype, and
`const fn zero` when the inner type is `usize`, so constants can be built. `#[derive(Display)]` forwards `Display` to
the inner value.

`Counter` is a blanket trait over every `Copy` newtype of `usize`: `zero`, `count`, `plus`, `incremented`
and `total`, saturating at the largest count. The orphan rule forbids a blanket `Add` or `Sum`, so these
are named methods. Derived code is not in the source, so `repeated_methods` does not see it. A
hand-written constructor on three types still fails, whatever their inner types are (an argument of the inner type
counts as `Inner`), and so does the same body of an operator, `Display`, `From`, `Default`, `Sum`, `AsRef`, `Deref` or
`FromStr` impl on three newtypes.

## Guards

A guard is a type that implements `Drop`. `guard_must_use` fails a guard without `#[must_use]`, also when the
attribute is behind `cfg_attr`. An impl binds to the type of that name in its own module, or through a type alias, or
else to the types of that name in its own crate; a crate with a same-named type that has no `Drop` is not flagged.

## Lifecycles

`states!` declares a lifecycle as a graph, in a module of its own.

    states::states! {
        pub mod pr {
            start Working { review -> Reviewing, abandon -> Abandoned }
            state Reviewing { approve -> Merged, rework -> Working }
            terminal Merged {}
            terminal Abandoned {}
        }
    }

    fn run<'id>(pr: pr::Working<'id>) -> pr::Finished<'id> {
        pr.review().approve().finish()
    }

    pr::Working::scope(run);

Each state is a `#[must_use]` type, and each edge a method that consumes it. `finish` exists on terminal states
only, and it is the only way to get `Finished`. `scope` gives the closure a fresh invariant brand `'id`, as
`GhostCell` does, and the closure must return `Finished` of that brand. A path that drops the state has no
witness to return, so it does not compile. A path that diverges, by `panic!`, `todo!`, `std::process::exit` or an endless
loop, returns nothing and is not caught.

The `compile_fail` doctests in `crates/states/src/compile_fail.md` prove this. `cargo test` runs them, and no
test code starts a process, though rustdoc itself runs `rustc` for each one. Stable rustdoc does not check the
error codes in them.

`state_graph` reads each `states!` graph in shipped code. Every state must be reachable from a `start` state and
reach a `terminal` state, and a terminal state has no outgoing edges. A graph it cannot read fails. It finds `states!` under a rename
(`use states::states as lifecycle`), inside other macros such as `cfg_if!`, and in `macro_rules!` bodies.

## Mutants

The `mutants` job in `ci.yml` runs on each pull request. It applies `cargo mutants` to the lines the pull
request changes, and tests each mutant with the whole workspace, because shared crates are tested through
the crates that use them (`.cargo/mutants.toml`). A mutant that no test catches fails the job. A mutant that makes the tests hang counts as caught, because no test can show more.

`mutants.yml` runs every mutant once a week, in eight shards, or on request. It reports in the run
summary and the shard artifacts. Missed mutants do not fail it; a shard fails only if the run itself
fails. No push starts it and no push cancels it.

## Fuzzing

The `fuzz` crate feeds the runner arbitrary bytes, arbitrary valid syntax, damaged valid syntax, deep nesting and long chains, and fails when the runner panics or overflows its stack. It is a workspace member but not a default member, so `cargo test`, `cargo clippy` and the PR gates never run it. It uses proptest rather than cargo-fuzz, which needs a nightly toolchain: proptest runs on the pinned stable toolchain, generates valid syntax from strategies, shrinks a failing input, and the targets need no coverage feedback to find a panic in a lint written over `syn`.

    FUZZ_SECONDS=60 FUZZ_CRASHES=/some/dir cargo test -p fuzz --test fuzz

Each target runs for `FUZZ_SECONDS` seconds (default 5) with fresh random seeds. With `FUZZ_CRASHES` set, the input being run is first written to `<target>-pending.rs` and `<target>-pending.role`, so an abort that no test can catch still leaves its input behind; a panic also leaves the shrunk input as `<target>-<role>-<hash>.rs`. The `fuzz` workflow runs every day on main for 15 minutes, fails the run on a crash and uploads that directory. A crash found becomes a named test in the crate that owns the fix, with the input kept in its `tests` directory.
