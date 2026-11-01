# meta-dev

Source lints for Rust repositories, run through one binary.

    lint <path>

## Install

Install the runner from a tag of this repository, with the tag as the version pin:

    cargo install --git https://github.com/Studio-Terza/meta-dev --tag v0.1.0 --locked lint

This puts the binary `lint` in `~/.cargo/bin`. Change the tag to change the version; a new tag is a new
version, and nothing updates on its own. `--locked` builds with the dependency versions in `Cargo.lock`.
The build needs Rust 1.99.0 or newer. The runner is not on crates.io and is not published there.
The install has no configuration file and the runner reads none.

In a checkout of this repository, `cargo run -p lint -- <path>` does the same without installing.

The runner finds the packages under the path (a directory with a `Cargo.toml` holding `[package]`,
listed by its workspace when one encloses it; hidden directories and `target` are skipped), reads each
package's crate roots and the modules they declare, runs all lints, prints the violations, and exits
non-zero if there are any. Whether a file is shipped code, a binary or a test is decided from its place
inside its package, so the verdict does not depend on where the checkout lives.

Lints: no_comments, no_free_fns, thin_main, vec_out_params, named_lifetimes, enum_predicates,
const_shape, type_shape, distinct_signatures, expression_flow, fn_shape, literal_names,
manual_iteration, module_nesting, named_numbers, no_forwarders, no_numbers_in_binaries,
no_parameter_bags, no_plaintext, primitive_boundary, private_fns, process_starters, raw_resources, public_surface,
repeated_methods, state_graph, guard_must_use, crate_graph, public_api_tests, no_test_only_api, no_ignored_tests,
near_duplicate_tests, mandatory_laws, rust_only, workflow_run.

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
per lint (`lint-crate-graph` among them), `lint-item-graph` (the item graph `crate_graph` reads), `lint-paths` (the paths a crate uses, resolved through its imports, shared by `process_starters` and `raw_resources`), `lint-fixtures` (shared test sources), `lint-rust-only` (the `rust_only` lint over every file of the tree), `lint-workflow-run` (the `workflow_run` lint over the workflows), `lint-runner` (the runner
library), `lint` (the binary), `lint-bool-methods` and `lint-paired-apis` (the report-only lists), `states` (the `states!` macro), `golden` (golden files for tests), `fuzz` (the fuzz targets, outside the default members), `laws` (law macros for tests), `newtype` (the newtype traits), `newtype-derive` (their derives), `gate-limits` (the CI wall-time gate), `reach` (what a change can affect), `ci` (the CI helper), `mutation-shard` and `mutation-report` (the mutation matrix), `test-log` (reads libtest output) and `text-lines` (lines of a text). Crates
stay split by lint because `crate_graph` flags a crate whose items fall into two loosely linked groups of
five or more, or whose cycle spans more than 40 items; its conductance floor is 19%.

## Rust only

`rust_only` fails a source file in another language: the file name has the extension of Python, shell, Batch,
PowerShell, Perl, Ruby, PHP, Lua, Go, Java, Kotlin, Scala, Swift, Objective-C, C, C++, C#, Dart, Haskell, Elixir,
Erlang, Clojure, OCaml, F#, R, Julia, Groovy, Zig, awk, sed, Tcl, Nix, make, notebooks, Vim script, Lisp, Scheme, Racket,
Nim, Visual Basic, Solidity, assembly, D, Pascal, Fortran, Crystal, V, Arduino, Starlark, JavaScript, TypeScript, HTML,
CSS, Vue, Svelte or Astro, in any case. Web assets (JavaScript, TypeScript, HTML, CSS, Vue, Svelte, Astro) pass below a
directory named `assets`, anywhere under the checked path; any other language fails there too. The message names the fix: rewrite the file in Rust, or for a web
language, move it under `assets` if it is a web asset. The lint reads every file under the path, hidden directories
included, so `.github/` is checked. A file with no extension is read for its first line: a `#!` line naming any interpreter other than `rust-script` or `cargo` fails it as an interpreted script. It skips `.git`, and `target` beside a `Cargo.toml`. Configuration files
(`toml`, `yaml`, `json`), Dockerfiles and Rust are not source files in another language. There is no allow-list.

`workflow_run` keeps logic out of YAML. It parses each file under `.github/workflows`, and every `action.yml` or `action.yaml`, as
YAML, so how a step is spelled does not matter. The value of a `run` key must be one line of at most
120 characters, with none of the shell operators `|`, `&`, `;`, `>`, `<`, a backtick, `$(` or a `${` expansion with
a modifier, and not start with `if`, `for`, `while`, `until`, `case`, `[`, `[[`, `!` or `(`. A value that holds several lines fails, however it is written. The
value of a `shell` key gets the same rules and must start with `bash`, `sh`, `pwsh`, `powershell` or `cmd`. A step that uses
`actions/github-script` fails if it has a `script`. A `uses` that starts with `./` must name a directory with an `action.yml` or `action.yaml`, or a
workflow file in the checked tree; that action is judged like a workflow. A workflow the lint cannot read or parse fails too. There is no allow-list: anything longer is a Rust
command.

The CI helpers are Rust too, and workflows only call them. `ci` has these commands:

    ci timed <measured> <stage> <program> [<argument>...]       time a stage into the measured file
    ci logged <log> <program> [<argument>...]                   run a program, copy its output to the log
    ci libtest-seconds <measured> <stage> <test output>         add up the libtest seconds
    ci reach <metadata> <event> <environment> <summary>         write the scope, say which scopes are empty
    ci tools <built directory> <tools directory>                build and copy reach, lint and ci
    ci mutants-changed <diff> <output> [<scope>...]             mutants on the changed lines, or say it skipped
    ci mutants-shard <shard> <output>                           one weekly shard of all mutants
    ci wall-times <repository> <run> <job>                      wall time per step, from the GitHub API
    ci show <file>...                                           print files, and say which are absent
    ci labels <base branch> <pull request>                      the golden-change and new-scenario checks
    ci shard-summary <shards> <summary>                         outcome counts of the weekly shards
    ci matrix <shards> <sources> <directory> <summary>          the mutation matrix report

`mutation-shard` and `mutation-report` read the cargo-mutants output
(`mutation-shard <mutants.out> <out.json>`, `mutation-report <shards> <sources> <report.md> <report.json>`). `ci`
starts processes only in its `ports` module.

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

`mandatory_laws` fails a `pub` type that has a conversion pair, or a hand-written `Ord` or `PartialEq`, without a law in a
test of the same package that names the type. A pair is `Display` with `FromStr`, `Serialize` with `Deserialize`
(also behind `cfg_attr`), `encode` with `decode`, or `to_x` with `from_x`, in inherent impls or any trait impl, in one
file or several. Each pair needs its own `roundtrip!`: one that names the type and, for method pairs, both methods.
The test title does not count as naming the type. Only types in a crate's public API are held to this: a private type's law is exercised through the public behaviour that uses it, and mutation testing covers that.

## Labels

`labels.yml` runs `ci labels`. It fails a pull request that changes a golden (including renaming one out of
`tests/goldens`) unless it carries the label `golden-change`, and one that adds, copies or renames a file into
`tests/fixtures` or `tests/e2e` unless it carries `new-scenario`. A person adds the label after reviewing the change.
The workflow reads label names only: it does not check who added a label. It compares the head of the branch with the
merge base of the head and the base branch, so a branch that is behind the base branch is not charged for goldens and
fixtures that are already on it.

## CI wall-time limits

`ci/limits.toml` holds a warn and a block level in seconds for each timed CI stage: clippy, test compile,
test run and lint. The levels assume a full run, so they apply on main only; a pull request is not judged
by them. The toolchain install has no limit, since it is not our code; it stays in the wall time per step
that main reports. `ci timed` times a stage; the test run is the sum of the `finished in`
times libtest prints (`ci libtest-seconds`), so compile time is not in it. The `limits` step runs `gate-limits`, which prints
each stage as ok, WARN or BLOCK and fails when any stage is above its block level. A stage with a limit
but no measurement, or a measurement with no limit, also fails. The levels come from seven cold runs
on `ubuntu-latest`: block just above the slowest run, warn just above the median.
No cache is used: caching was measured and did not pay for its save time.

## Reach

A pull request pays only for what it can affect. `ci reach` lists the files the pull request changes and
gives them to `reach`, with the output of `cargo metadata --no-deps`:

    reach affected <metadata> [<changed file>...]
    reach fan-in <metadata>

A file inside a crate reaches that crate and every crate that depends on it, directly or not, through
normal, dev or build dependencies. `README.md` and `specs/` reach nothing. Any other file reaches every crate:
that covers `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `clippy.toml`, `.cargo/`, `.github/`, `ci/`, the
files of a deleted crate and anything not yet known. A file `reach` cannot place fails closed.

`affected` prints the share of crates reached and the scopes CI uses, and the run summary of each pull
request shows them. Clippy, the tests and mutants run on that scope, and are skipped when nothing is
reached. A skip is not silent: for each scope that is empty, `ci reach` prints `skipped: <scope> is empty, so the
steps that use it do not run` in the step and in the run summary, and `ci mutants-changed` prints that mutants were
skipped when it gets no scope. The lint step and `cargo fmt` stay whole-repo: lints such as `crate_graph` and `repeated_methods`
cannot judge a slice. Main ignores all this and always runs everything, with `--workspace`.

`fan-in` lists how many crates depend on each crate, directly or not, most first. How far a change reaches
is the measure of tangle, so the report on main prints it.

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
are named methods. Derived code is not in the source, so `repeated_methods` does not see it, and neither does it
count the impl of a trait (an operator, `Display`, `From`, `AsRef`, `FromStr`). A hand-written constructor or other
public method of one shape on three types still fails, whatever their inner types are (an argument of the inner type
counts as `Inner`); the fix is `AsRef<T>` for a stored field or `From<&T>` for a computed value, on a newtype, or a derive.

Text is typed: `primitive_boundary` fails a bare `&str` or `String` in a public function signature or field, except in trait impls and inside `ports`; use a newtype, with `Display` at the render edge and `FromStr` at the parse edge.

Data read from a file has structure, so `no_plaintext` fails a type that derives `Deserialize` and has a field of `String`, `&str`, `Cow<str>` (also inside `Option`, `Vec` and other generics) or a newtype that wraps one without validation. A newtype around text passes only if its deserialization validates: `#[serde(try_from = ...)]` on the type, or a hand-written `Deserialize` that calls `FromStr` (`from_str` or `parse`). The fix is typed parts (an enum, a number, a `PathBuf`) or a validating newtype such as `text_lines::OneLine`. A type with `try_from` is not checked field by field. There is no allow-list.

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
request changes, and tests each mutant with the crates the pull request reaches, because shared crates are
tested through the crates that use them. A mutant that no test catches fails the job. A mutant that makes the tests hang counts as caught, because no test can show more.

`mutants.yml` runs every mutant once a week, in eight shards, or on request, each against the whole
workspace (`.cargo/mutants.toml`). It reports in the run
summary and the shard artifacts. Missed mutants do not fail it; a shard fails only if the run itself
fails. No push starts it and no push cancels it.

## Fuzzing

The `fuzz` crate feeds the runner arbitrary bytes, arbitrary valid syntax, damaged valid syntax, deep nesting and long chains, and fails when the runner panics or overflows its stack. Its test target needs the `campaign` feature, so `cargo test --workspace` and the PR gates never build or run it; only the `fuzz` workflow does. It uses proptest rather than cargo-fuzz, which needs a nightly toolchain: proptest runs on the pinned stable toolchain, generates valid syntax from strategies, shrinks a failing input, and the targets need no coverage feedback to find a panic in a lint written over `syn`.

    FUZZ_SECONDS=60 FUZZ_CRASHES=/some/dir cargo test -p fuzz --features campaign --test fuzz

Each target runs for `FUZZ_SECONDS` seconds (default 5) with fresh random seeds. With `FUZZ_CRASHES` set, the input being run is first written to `<target>-pending.rs` and `<target>-pending.role`, so an abort that no test can catch still leaves its input behind; a panic also leaves the shrunk input as `<target>-<role>-<hash>.rs`. The `fuzz` workflow runs every day on main for 15 minutes, fails the run on a crash and uploads that directory. A crash found becomes a named test in the crate that owns the fix, with the input kept in its `tests` directory.

Found so far: a method chain of ten thousand links overflowed the stack of the main thread, because `syn` prints and walks a left-nested tree recursively. `Cli::report_to` now does its work on a thread with a 256 MiB stack, which is address space and not memory, and `a_twenty_thousand_link_method_chain_is_linted_without_overflowing_the_stack` holds it.
