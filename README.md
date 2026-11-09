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
inside its package, so the verdict does not depend on where the checkout lives. A file whose brackets or angle
brackets nest more than 256 deep is refused as a parse failure before it is parsed, since no handwritten code does
that and the parser would run out of stack or time.

Lints: no_comments, no_free_fns, thin_main, vec_out_params, named_lifetimes, enum_predicates,
const_shape, type_shape, distinct_signatures, expression_flow, fn_shape, literal_names,
manual_iteration, module_nesting, named_numbers, no_forwarders, no_numbers_in_binaries,
no_parameter_bags, no_plaintext, primitive_boundary, private_fns, process_starters, raw_resources, public_surface,
repeated_methods, state_graph, guard_must_use, crate_graph, public_api_tests, no_test_only_api, no_ignored_tests,
no_test_io, fakes_at_ports, long_exact_strings, repeated_shapes, mandatory_laws, arbitrary_newtypes, bool_methods,
effects_in_ports, no_early_exits, one_edge, unconsumed_traits, rust_only, workflow_run, owned_dependencies,
key_dependency_roles, effects_in_fillers, counted_calls, hidden_sqlx, store_queries.

`repeated_shapes` keeps each shape written once. A function or method body, closure, block or match expression of 40
tokens or more may not repeat anywhere in the workspace, shipped code, tests and examples alike, once its local names
and literals (`true` and `false` included) are erased; callees, paths, field accesses and capitalised types stay, while
struct-literal and pattern fields and lower-case primitive types other than `u8`, `usize`, `bool`, `str` and `char` are
erased like locals. A group nested inside a
larger group is not listed again. Each group counts the literal positions at which its copies do not all agree, or says
that they are identical up to local names. The message names the fix by where the
copies live: in tests, make them rows of one `laws::table!` or state the property they share with a law; in shipped code,
extract the shape once, write it generic over an existing trait, or derive it; in both, the test re-implements shipped
logic and should test through it.

Every run also prints the test budget: tests, files, lines, test lines and process starts per package, counting law macros as tests. Process starts
are counted everywhere, behind `ports` or not; only those outside `ports` are violations.
It is report only, no number blocks. The run also lists public methods that form a pair such as
`start` and `stop` (`paired_apis`), also when the two sit in different impl blocks of one type in a crate, with each entry saying to write a guard. That list is report only too,
so it is outside the lints above and never changes the exit code. It also lists each `.clone()` call in shipped code outside a module named
`boundary` or a crate whose name ends in `boundary` (`clones`), with the fix: borrow, take a slice or `&str`, name a
lifetime, or move the value; report only as well. Clippy's `redundant_clone`, `implicit_clone`, `unnecessary_to_owned`,
`clone_on_ref_ptr`, `needless_collect` and `needless_pass_by_value` warn across the workspace, and CI denies warnings.

`primitive_boundary` keeps primitives at the boundary, by convention as `ports` keeps effects. In shipped code, `bool`, the
integers, the floats, `char`, `String` and `str`, and tuples of them, may appear in a function signature (parameters, return
type, generics and bounds, nested in `Option`, `Vec`, slices, references or closures) or in a struct, enum variant or union
field only inside a module named `boundary` or a crate whose name ends in the word `boundary`. Allowed everywhere: a newtype
(a struct with one unnamed field) and its own `new`, `try_new` and `into_inner` with one input, and trait impls, since the
trait fixes the signature (`Display`, `FromStr`, `From`, `TryFrom`, `Debug`, `Hash`, `Ord` and the like); the trait
declaration is read instead. A unit return is not flagged, nor are consts, statics and types inside bodies. Wire forms such
as `serde_json::Value` and `toml::Value` are flagged the same way; a std trait such as `std::any::Any` is never a primitive
or wire form, whether written as `dyn`, `impl` or a bound. Text handling (`trim`, `format!`, `to_string` and the like)
is allowed in a door impl (`FromStr`, `TryFrom`, `Display`, `AsRef`, `Borrow`, `Deref`), on a type that has one, and in a
boundary. The message names the fix: take and return a domain type, or move the code into a module named `boundary`.

`unconsumed_traits` refuses a local trait with methods that nothing in shipped code consumes: no bound (generic, where
clause or supertrait), no `impl Trait` or `dyn Trait`, and no call to one of its provided methods. Delete the trait and
keep its methods inherent, or give it a consumer.

`one_edge` keeps one transition per pair. In shipped code, a set of domain input types reaches one domain output type
by one written edge: a function, a method (the receiver is an input) or a trait impl, read through references, `Option`,
`Result` and collections to the type inside. A trait impl also takes its implementing type when no input or output carries
it, so impls of one trait on different types are different edges, while a `From`, `TryFrom` or `FromStr` impl meets a
function between the same pair. An operation whose inputs and output are all one type (`A, A -> A`) belongs to that
type's algebra and is judged by its laws, not compared. Type names resolve through their path or `use` import; a name declared in several crates
and not imported, or twice in one crate, is a leaf. Edges touching a bare primitive, an interface or a foreign type are not
compared, nor are derived lifts (a map, field focus or match routing) and store operations. The message names the fix:
keep the standard conversion, or one transition, and route the others through it; delete a hand-written copy of a map,
filter or fold over the element transition; where the transitions mean different things, give each its own output type.

`distinct_signatures` keeps the methods of one type apart. Every inherent method of a type, in any file of its crate, is
read on the type graph: its inputs (the receiver first) and its output as the domain types, interfaces, primitives and
black boxes they name, read through references, generic arguments and tuples, with whether each is optional, fallible or
many (a collection, a slice or an `impl Iterator`), and whether the method mutates self or a parameter. Two methods of one
type that read alike hide a missing type or type parameter. Trait impls are fixed by their trait and are not compared, a
method written once per `cfg` counts once, and a type name declared twice in one crate is compared within its own file.
The message names the fix: distinguish the types they take or return, or keep one method and route the other through it.

`effects_in_ports` keeps effects in ports. In shipped code, a call into files, the environment, the clock, the network,
processes, randomness or stdio (`std::fs`, `std::env`, `SystemTime::now`, `Instant::now`, `thread::sleep`, `std::net`,
`process::Command::new`, `process::exit`, `rand`, `io::stdout` and the like, and `println!`, `eprintln!` and `dbg!`) is
allowed only in a crate whose name ends in the word `ports` or a module named `ports`, read through imports as
`process_starters` reads them. The shared ports are `file-ports` (`Files`, with `Disk`) and `console-ports`
(`Process` for a binary's arguments and streams, `Environment` for variables); code takes the port as a parameter, and
its tests pass the port's in-memory fake, `Memory` for files and `Variables` for the environment. Fixture trees on disk
load into a `Memory` through `golden::fixtures!`, as recorded output is checked through `golden!`. The `--report` run still counts effect sites per crate, in
ports and outside.

`bool_methods` blocks any function in shipped code that returns `bool`, whatever its name or visibility, including trait
declarations and functions inside bodies. Only a trait impl may, since the trait fixes its signature (`PartialEq::eq` and the
like). Return an enum to match on, an `Option` or `Result` carrying the value, or a proof type such as `NonEmpty`.

`narrowed_returns` lists, in the `--report` run only, every function returning `bool`, `Result<(), E>` or `Option<E>`
where `E` is an error type, read through the code model: return types resolve through imports, globs and aliases (crate
aliases and `fmt::Result`, `io::Result`, `thread::Result`), so a crate type named `Result` or a generic named like an
error is not flagged. An error type is a crate type with an `Error` impl or a `thiserror` derive, a crate type returned as
the `Err` of a `Result` outside trait impls, or `io::Error` and `fmt::Error`. Trait impls, `#[test]` functions and the
crate's root `main` are exempt; test-scope helpers and inherent `fmt::Result` helpers are flagged. It stays report-only,
beside the blocking `bool_methods`, until the backlog is fixed; then it joins the gate and replaces `bool_methods`.

`no_early_exits` blocks the `return`, `break` and `continue` keywords in shipped and test code alike: function bodies,
const fns, trait impls, closures, labeled blocks, `#[cfg(test)]` modules, files under `tests` and macro bodies. A body
leaves through its tail value or through `?`, which stays allowed. The message names the fix: yield the value from an
`if` or `match` expression, drive the iteration with `find`, `take_while`, `try_fold` or `position`, or split it into
smaller functions. There is no function-size lint.

The linear flow lints of `lint-linear-flow` block in shipped and test code alike; they replace the hidden-states
report. A body is let bindings plus pipeline expressions, and the one non-linear
form is a pure router: a function whose whole body is one `match` or `if let`. `linear_bodies` lists every `if`, `if let`,
`match`, `while`, `for`, `loop` and let-else in shipped and test code, closures and macro bodies included, except the
`match` or `if let` that is a function's whole body; const fns are skipped, since combinators are not const.
`pure_routers` reads that whole-body `match` or `if let`: no guard, every pattern a variant of an enum, `Option` or
`Result` (no literal, range, tuple or slice, nested or not), and each arm a single call or value (a call or method chain,
path, literal, constructor, macro, reference, field, index, cast, closure, `?` or `.await`, or a block holding one of
these). `plain_conditions` lists every `&&`, `||` and `!`, inside conditions and out. There are no thresholds. The
fixes: combinators on `Option`, `Result` and iterators; extract an arm's logic into its own linear function; name the
condition as an enum or a predicate type. Shared routers over syn and token shapes live in `lint_core::Syntax`. The
`--report` run keeps `flow_shapes`, which counts per crate the functions that are a pipeline, a pure router, or
neither.

The `--report` run also prints the containment reports of `lint-transition-graph`, report only until the owner has
reviewed them. The crates, modules and files are a nested clustering of the type and trait graph: types and traits are
linked by fields, variants, bounds, trait impls and the signatures of their methods, read through the same facts as
the transition graph. One engine (`Containment`, with a `ClusterLint` per check) runs every check at the
crate and the module level: `connected_clusters` (each crate and each directory module is one connected piece),
`acyclic_quotients` (the crates of the workspace, and the own file and child modules of each crate and module, collapsed
to single nodes of the definition graph of fields, variants and bounds, form no cycle; signatures and bodies may cycle, as functions do), `interface_by_consumers` (a cluster's exports, the types and
traits used from outside it, form one group when linked by a shared outside user),
and `lowest_cluster` (each type and trait sits in the deepest cluster that holds all its users). There are no thresholds.

The same run prints `repeated_routes`, report only: call sub-sequences of at least three hops through at least two distinct functions that repeat in at least two different bodies, with their locations. A repeat inside one body is not counted, and a route whose ends are already joined by a function is noted. A call is resolved by the type that owns it: `self`, a typed parameter, the output of an earlier resolved call, a type path or `Self`; a bare or module path call goes to a free function. It resolves only when exactly one workspace function of that name has that owner; method calls on any other receiver, and calls into std, are left out. The header states these settings and the share of calls not resolved.

The same run prints `parallel_structures`, report only: domain types whose moves (written, operation, port, lift and
store edges, with names erased) are the same up to one consistent renaming of the types they reach, checked one hop
deep, with each type located at its declaration. Trait impls are the shared interface of their fillers and are set
aside as evidence; where two types fill one interface, their impls of it must have the same bodies with names and
literals erased. Algebra laws must match where both sides declare them. The header names these settings.

The same run prints `normalised_shapes`, report only: the `repeated_shapes` check with every name erased (callees,
paths, fields and types as well as locals) and literals erased, floor 40 tokens, nested groups dropped. Groups that
`repeated_shapes` already blocks are listed again. It is the detector for normalisation before comparison, and the
owner decides whether it is promoted.

Every lint implements `Lint` and takes the same files. Every message names its fix: `Violation::new` takes a
`Finding`, whose `Display` states the problem and whose `fix` returns a `Fix`, so a message without a fix does not
compile, and each violation prints as `path:line: problem; fix`. A constant message is a `Message` of a `Problem` and a `Fix`. Limits are constants inside each lint, not
settings, and there is no per-repository configuration, switch or baseline. Where something is sometimes
legitimate, the lint gates where by naming convention: starting a process (`process_starters`) and calling
`fs::remove_dir_all`, `mem::forget` or `env::temp_dir` (`raw_resources`) are allowed only in a crate whose name
ends in the word `ports` (`meta-ports`, not `meta-reports`) or a module named `ports`. Both lints read paths through
`use` items in any order, renames, type aliases, glob imports, re-exports across the files of a crate, qualified
`<T>::f` calls and macro bodies, so `matches!(..., Ok(s) if s.success())` is read like any other call. Inside `ports`,
a raw call must sit in an impl of a type that implements `Drop`: the guard that does the cleanup. Code a platform requires in its own language (XCUITest Swift, say) is allowed only under a directory named `platform`, and only as test code (`rust_only`). In a module named `boundary`, or a crate whose name ends in the word `boundary`, only: casts (`const_shape`), bare numbers
(`named_numbers`) and bare primitives (`primitive_boundary`). Files under
`tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

`no_test_io` keeps effects out of tests. In a file under a package's `tests` directory it fails a path in an expression
or a macro body that resolves, through imports as `process_starters` reads them, into `std::fs`, `std::net`, `std::env`,
`std::process`, `std::os::unix::fs`, `std::os::unix::net`, `std::os::windows::fs` or the `tempfile` crate, or that names
`std::io::stdin`, `stdout` or `stderr` or a real filler of the shared ports: `file_ports::Disk` and `console_ports::Process`,
also through `golden`. `std::process::ExitCode` is a value and passes; so do types in signatures and `env!`, which is read
at compile time. Real I/O in test code is allowed only in the tests of a crate whose name ends in the word `ports`, where a
port's tests exercise its real filler; a module or test target named `ports` in any other crate no longer exempts it.
Everywhere else a test goes through the port's in-memory fake or a library API such as `golden!` or `golden::fixtures!`,
and the message names that fix. A method call on a value, such as `path.exists()`, is not resolved and is not read. Starting
a process in test code still also needs what `process_starters` asks for.

`owned_dependencies`, `key_dependency_roles`, `effects_in_fillers`, `counted_calls` and `hidden_sqlx` (crate `lint-port-dependencies`) keep the
port framework the only way to an effect. Ownership is declared, never guessed: a crate owns a package by naming it in
`[package.metadata.port] owns = ["logos"]` in its `Cargo.toml` (`grammar` owns `logos`, `logos-codegen` and
`logos-derive` and only re-exports `logos`, which the `grammar-<format>` lexer crates reach as `grammar::logos` with
`#[logos(crate = grammar::logos)]` on each derive, while the `grammar-cited` lexer's `Cited` is shared by two other lexers and `grammar-unspelled` holds the `Unspelled` refusal that every lexer's checked newtype and `FromStr` shares; `store-ports` owns `sqlx`, `sqlx-macros`, its drivers, `tokio` and `tokio-stream`; `store-ports-macros`
owns `sqlx-macros-core`). An owning unit is a crate with a `[package.metadata.port]` table together with every workspace
proc-macro crate it depends on, so `store-ports` and `store-ports-macros` own jointly, and a proc-macro crate two ports
depend on belongs to both units. The gate reads the dependency graph from `cargo metadata --format-version 1 --locked`,
through the `Cargo` port of `cargo-ports`, and refuses to run when that fails; packages are told apart by their resolved
identities. A package named in `owns` belongs to the declaring unit alone: `owned_dependencies` fails every other crate
that depends on it directly, another unit whose closure holds it included, and names the declaring crate. An `owns` entry
that resolves to no package the declaring unit depends on, directly or not, fails at its manifest line. The closure rule
covers the rest: every package reachable from a package a unit owns belongs to that unit, and a package no unit names is
depended on directly only by the units whose closure holds it; every other workspace crate fails where it depends on it,
in `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` or their `[target.*]` forms, also when renamed with
`package =`. It points at the manifest line and names every unit whose closure holds the package. So a direct
`libsqlite3-sys` dependency outside `store-ports` fails although no crate names it in `owns`. A package of the
closure that other crates may depend on directly is declared by the unit in `[package.metadata.port] shared = [...]`:
`store-ports` shares `proc-macro2`, `quote`, `serde`, `serde_core`, `serde_json`, `syn` and `thiserror`, and `grammar`
shares `proc-macro2`, `quote` and `syn`. Nothing else is shared: a crate that needs an unshared package calls an owner's
API. `effects_in_fillers` judges a crate that declares a `#[port]` (or `#[port_core::port]`) trait: every
effect site, in the vocabulary of `effects_in_ports`, must sit inside an `impl <Trait>Filler for ..` block of a port trait
that crate declares or an `impl Release for ..` block (a held resource's release), never in an inherent method, a free
function or another trait impl. Test code is skipped as
`effects_in_ports` skips it, and a crate with no `#[port]` trait is not judged. `counted_calls` fails the identifiers
`counted_call` and `counted_acquire` anywhere in source, macro bodies included, outside `port-core` and `port-derive`; the
code `#[port]` expands to is not source, so only the generated fillers count calls and acquisitions. `hidden_sqlx` fails
the identifier `__sqlx` anywhere in source outside `store-ports` and `store-ports-macros`: `store_ports` re-exports sqlx as
`__sqlx` only so that the code `store_ports::query!` and `store_ports::migrate!` expand to can name it.

`key_dependency_roles` keeps each key dependency in its role, so a heavy package stays out of crates that do not need
it. A port that names `[package.metadata.port] dependents = "lexer"` (as `grammar` does) is depended on directly, in any
dependency table, only by crates that declare `[package.metadata.lexer]`. A lexer crate declares the one format it lexes
with `format = "<name>"`, no two lexers lex the same format, and a lexer is a leaf: through normal edges it depends only
on a port whose dependents are lexers, on other lexers, and on packages whose normal closure holds no package a port
owns. A port declares who expands its macros with `expanders = { role = "store", macros = ["query", "migrate",
"Columns"] }` (as `store-ports-macros` does), and shipped code that invokes one of those macros through any crate of the
unit (`store_ports::query!` or `store_ports_macros::query!`, or the name imported from either, or `#[derive(Columns)]`,
also inside `cfg_attr`) belongs to the unit or to a crate
that declares `[package.metadata.store]`, as `design-store` does; test code is skipped. A workspace proc-macro crate whose
normal closure holds a package a port owns belongs to that port's unit, or reaches the package only through crates that
declare the port's `dependents` role, as `port-derive` reaches `logos` only through `grammar-ident`, so a proc-macro
never drags a heavy dependency into the host build of every crate that uses it. Every lexer crate, the port that names
lexers as its dependents, and every workspace crate that only those crates depend on (as `grammar-unspelled` is) starts
its `lib.rs` with `#![deny(clippy::wildcard_enum_match_arm)]` (or `forbid`), the crate-level deny the single `grammar`
crate carried before the lexers were split out of it; a deny under `cfg_attr` does not count. Each finding points at the
manifest line of the dependency, the format, the macro call, or the first line of the crate root missing the deny.

The store is reached through `store_ports::query!`, never through sqlx. `store_ports::query!(Row, r#"…"#, id: NodeId)` reads
rows of a `#[derive(Columns)]` struct for `Statements::rows` or `Statements::one`, and `store_ports::query!(r#"…"#, id: NodeId)`
writes for `Statements::written`; sqlx checks the SQL against the schema as its own `query!` does. Every bound argument and
every field of a row is a `store_ports::Column`: a newtype over an integer, float, text or bytes, an enum with
`#[derive(Named)]`, a type with `#[derive(newtype::Spelled)]` that is stored as its `Display` and read back through
`FromStr`, or an `Option` of one. A checked newtype (`#[newtype(checked)]`) is stored as its inner value and read
back through `newtype::Guarded`, so a stored value its check refuses fails the read; `grammar-output`'s `Bare`
and `Quoted` fields read it the same way. A bare `i64`, `&str` or `String` is not a column and does not compile.

A store's schema is a `store_ports::Schema`: its SQL migrations, and the typed Rust rewrites, each a `Batch` due when
the migration of a given version is pending. Opening a store applies every pending migration and then every due rewrite,
in version order, in one transaction: a store whose applied migrations do not match their declared checksums, a failing
migration or a failing rewrite leaves the store exactly as it was. A rewrite runs only on the latest schema, the one
`.sqlx` checks its queries against, so a later migration that renames a table or column a rewrite reads stops its query
compiling instead of failing on an old store. A rule that Rust already owns, such as how the segmenter cuts a message
into segments and sheds blanks, is never restated in SQL: `Resegmented`, due with migration 20261108000012, re-derives
every message's segments through the same segmenter ingest runs, rewriting each segment's shape and body and adding and
removing segments, so an old store holds the segments a fresh ingest of the same transcript would. Migration
20261108000012 keeps the search index in step with every update and delete of a segment. Every row that points at a
segment does so through a foreign key into `segments`, and the rewrite never lists those tables: it deletes each stored
segment the segmenter would change or drop through `Statements::released`, which answers `Released::Referenced` where a
foreign key refuses the delete, so SQLite checks every foreign key `pragma_foreign_key_list` reports, a table added
later included. A re-derivation that would change the shape or body of a segment any row points at, or drop it, refuses
the open with `StoreFailure::Refused`, naming every such segment as `message#ordinal`, and the store is left as it was.
Segments nothing points at are still re-derived. Migration 20261109000001 gives `mechanical_links` generated
`source_ordinal` and `target_ordinal` columns, read from the ordinal of a segment anchor (null for a message anchor),
with foreign keys into `segments`, so a `cites` or `answers` link holds its segment like any annotation and its ordinal
cannot disagree with its anchor. The migration copies the stored links with those keys deferred, and the rewrite due with
it, `Anchored`, reads `pragma_foreign_key_check` for `mechanical_links`: an old store holding a link whose segment it does
not hold refuses the open with `StoreFailure::Refused`, naming every such link as `source relation target`, never with a
bare foreign key failure. Ingest links over every message of a session, earlier ones re-segmented in memory, so a link
can name a segment of an earlier message that the store holds differently or not at all, such as one stored under an
older segmenter. Before writing, `Holding` checks every segment each link names: the row the store will hold, the
stored one or the one written now, must be the row the segmenter derives from that message. A link that fails is not
written and ingest prints it as `refused link source relation target`; the rest of the session is written and its cursor
advances. `every_foreign_key_into_segments_is_immediate_and_neither_cascades_nor_sets` reads every foreign key into
`segments` through a checked query over `pragma_foreign_key_list` and fails on an ON DELETE or ON UPDATE action or a
table declaring `DEFERRABLE`, since the rewrite's refusal relies on SQLite refusing the delete at once.

`store_queries` (crate `lint-store-queries`) fails a `store_ports::query!` whose SQL is not written exactly as `sqlformat`
formats it with its default options, starting at column 0 on the line after `r#"` and ending with a line break before `"#`.

`fakes_at_ports` keeps test doubles at ports. In a file under a package's `tests` directory, a struct, enum or union
whose name has the word `Fake`, `Stub`, `Mock`, `Spy` or `Dummy` among its camel-case words (`FakeClock`, `HTTPStub`, not
`Fakery` or `Spyglass`) must implement a port: a trait whose path, read through the `use` items of its test target as
`lint-paths` reads them, passes through a module named `ports`, starts at a crate whose name ends in the word `ports`, or
names a trait that crate declares in a `ports` module or a crate whose name ends in ports (so a re-export such as `ci::Ports`
counts). Besides ports it may implement only std traits: a path that starts at `std`, `core` or `alloc`, or a bare name
of a prelude trait such as `Clone`, `Default` or `Drop`. Derives and inherent impls are not trait impls and pass. The impls
of a double are matched by name within its test target, across its files. A double with no port fails at its declaration,
and any other trait impl fails at the impl; the message names the fix: implement a port, or rename the type.

`long_exact_strings` keeps long text out of assertions. It fails an `assert_eq!` or `assert_ne!`, an `assert!` whose condition is
`==` or `!=`, or a row of `laws::table!` (also when `table` is imported or renamed), when a compared side, or an expected value of
the table, is a string or byte string literal holding a line break (a `\n` escape or a real one), or is built with Debug
formatting: `format!` or `format_args!` whose format string has a `{:?}`, `{:#?}` or `{name:?}` placeholder. The compared side
is read through references, `to_owned`, `to_string`, `into`, `clone` and the like, `Some`, `Ok`, `Err`, `String::from`, arrays,
tuples, struct fields, `concat!` and `vec!`, and through a local `let` or a `const` of the file that it names. Arguments of
other calls are inputs and are not read, nor is the failure message of an assertion. The definition is structural: there is
no length threshold. The message names the fix: compare a golden with `golden!`, or assert on the parts, such as the lines.

A check that cannot run under plain `cargo test`, such as one that drives a browser or needs its own time budget,
goes in an `examples/` target that the gate runs, never behind `#[ignore]`. `no_ignored_tests` does not read
`test = false` on an `[[example]]`, since an example is not a test.

`no_test_only_api` also fails a visible `fn` or enum variant that no shipped code names but a test does, even with no
`cfg(test)` gate or `#[doc(hidden)]`. It matches by name across the files it is given, so a name that shipped code also
uses elsewhere passes.

Crates: `lint-core` (the `Lint` trait, reports, sites, violations, source files), `lint-layout` (names, matching, manifests, failures, findings and their fixes), `lint-placement` (where a file sits: package, crate root, module path), `lint-roots` (package and workspace discovery), `lint-reader` (reads a tree into source files), one `lint-<name>` crate
per lint (`lint-crate-graph` among them), `lint-item-graph` (the item graph `crate_graph` reads), `lint-paths` (the paths a crate uses, resolved through its imports, shared by `process_starters`, `raw_resources` and `no_test_io`, whose imports `fakes_at_ports` reads too), `lint-fixtures` (shared test sources), `lint-rust-only` (the `rust_only` lint over every file of the tree), `lint-workflow-run` (the `workflow_run` lint over the workflows), `graph-query` (the `lint query` command), `lint-runner` (the runner
library), `lint` (the binary), `lint-arbitrary-newtypes` (the `arbitrary_newtypes` lint), `lint-port-dependencies` (the `owned_dependencies`, `effects_in_fillers`, `counted_calls` and `hidden_sqlx` lints), `lint-store-queries` (the `store_queries` lint), `lint-bool-methods` (the bool lint), `lint-linear-flow` (the linear flow lints), `lint-paired-apis` (the report-only list), `states` (the `states!` macro), `golden` (golden files for tests), `fuzz` (the fuzz targets, outside the default members), `laws` (law macros for tests), `newtype` (the newtype traits), `newtype-derive` (their derives), `gate-limits` (the CI wall-time gate), `reach` (what a change can affect), `workspace-metadata` (the crates, dependencies and lib targets of `cargo metadata`), `ci` (the CI helper), `mutation-shard` and `mutation-report` (the mutation matrix), `test-log` (reads libtest output), `text-lines` (lines of a text), `store-ports` (the store port: the `Store` trait with per-type `read` and `write` ops and `lock`, its `Sqlite` filler over sqlx, `StoreLock`, the first held resource, and the `Column` and `Columns` traits), `store-ports-macros` (`store_ports::query!`, `store_ports::migrate!` and `#[derive(Columns)]`, over `sqlx-macros-core`, whose version moves in lockstep with sqlx as `[package.metadata.lockstep]` records), `design-store` (the design store's schema, queries and batches), `file-ports` (the file system port: the `Files` trait, `Disk` and the in-memory `Memory`), `cargo-ports` (the `cargo metadata` port: the `Cargo` port trait, `Toolchain` and the in-memory `Recorded`) and `console-ports` (the process's arguments, standard streams and environment: `Process`, and the in-memory `Variables`). Crates
stay split by lint because `crate_graph` flags a crate whose cycle spans more than 40 items.

## Query

Before writing a new trait, port or store, ask what already exists:

    lint query <term>... [--role '<fn signature>']... [--top <n>] [--in <path>]...
               [--defined <name>]... [--uses <name>]... [--users <name>]... [--implementers <name>]... [--cluster <name>]...
               [--exports <crate|module>]... [--path <from> <to>]...

It reads the shipped code under each `--in` path (the current directory by default) through the same extraction the
transition graph uses (`type-facts` and `transition-facts`, no second parser) and lists the traits, generic impls, ports
and stores that match, best first, each with its crate, site and implementers (or the traits it implements). A term
matches a camel-case or snake-case word of a name, a trailing plural `s` aside: 3 points in the name itself, 1 in a
method name or an implementer. A role is a Rust signature whose names are erased: the receiver (none, `self`, `&self`,
`&mut self`), the arguments that are the type itself, the arity, and the return shape (nothing, `Self`, `Option`,
`Result`, many, or another value); a method agreeing on all four adds 2 points and on three adds 1, and the line under
the match shows the method that fits. A port is a type or trait behind the `ports` gate, a store one behind `store`.
The command blocks nothing and exits non-zero only on malformed arguments.

    $ lint query index --role 'fn f(&self, q: &Query) -> Vec<Hit>' --in crates
    score 5: trait Index in search at crates/search/src/lib.rs:1; implemented by TextIndex
        search fn(&self, _) -> many fits fn(&self, _) -> many
    score 2: generic impl Lookup for T in search at crates/search/src/lib.rs:14
        find fn(&self, _) -> many fits fn(&self, _) -> many

`--defined`, `--uses`, `--users`, `--implementers` and `--cluster` ask about one type or trait by its bare name: where it
is defined, what it uses, what uses it, what implements it, and the module and lowest cluster it sits in with its
siblings. A name shared by several items is answered once under each full path. The answers come from the containment
view, so a field, a bound, an impl, a signature or a body each count as a use; only types and traits are indexed, not
functions. `--implementers` also lists generic impls and impls on types outside the index (`impl<T: Clone> Priced for Vec<T>`,
`impl Priced for u32`, and slices, tuples, `dyn` and fn pointers such as `impl for [u8]`) as `impl for <type>` with their site; an impl in another crate for an indexed type counts as that type's implementation. The questions combine with each other and with terms and roles.

`--exports` takes a crate or module name (crate names with underscores, such as `graph_query`) and lists the types and
traits used from outside it, each with its outside users. `--path` takes two names and prints the shortest chain of uses
between those types or traits, or `none`; it follows uses between types and traits, not function calls. `--top` caps
the exports and the path steps shown, as it does the other lists.

    $ lint query --users Cents --implementers Priced
    users of Cents:
      shop::Cents at crates/shop/src/lib.rs:5
        Line at crates/shop/src/till.rs:2
        Priced at crates/shop/src/lib.rs:2
    implementers of Priced:
      shop::Priced at crates/shop/src/lib.rs:2
        Line at crates/shop/src/till.rs:2

## Rust only

`rust_only` fails a source file in another language: the file name has the extension of Python, shell, Batch,
PowerShell, Perl, Ruby, PHP, Lua, Go, Java, Kotlin, Scala, Swift, Objective-C, C, C++, C#, Dart, Haskell, Elixir,
Erlang, Clojure, OCaml, F#, R, Julia, Groovy, Zig, awk, sed, Tcl, Nix, make, notebooks, Vim script, Lisp, Scheme, Racket,
Nim, Visual Basic, Solidity, assembly, D, Pascal, Fortran, Crystal, V, Arduino, Starlark, JavaScript, TypeScript, HTML,
CSS, Vue, Svelte or Astro, in any case. Web assets (JavaScript, TypeScript, HTML, CSS, Vue, Svelte, Astro) pass below a
directory named `assets`, anywhere under the checked path; any other language fails there too. The message names the fix: rewrite the file in Rust, or for a web
language, move it under `assets` if it is a web asset. The lint reads every file under the path, hidden directories
included, so `.github/` is checked. Below a directory named `platform`, source of any language passes only as test code: a directory or file name before its first dot that is `test` or `tests` in any case, or ends in `Test` or `Tests` (`UITests`, `FlowTests.swift`); other code there fails. A renamed or disguised file is source: a double extension that hides a source extension (`run.py.bak`, `Flow.swift.txt`) fails like the source itself, and so does a file whose content is source in any name: a first line `#!` naming any interpreter other than `rust-script` or `cargo` (`#![` is a Rust attribute), or lines at the start of a line that clearly are Python (`def f(...):`, `import`, `from x import y`), Swift (`import XCTest`, `import SwiftUI`, an `XCTestCase` class), shell (`set -e`, `if ...; then`, `for ...; do`) or C (`#include`), weighed so that one clear line or two weak ones suffice. Content is read from the first 8 KiB, and lines inside fenced code blocks of a document are ignored, so prose and data pass. Rust files are not read. It skips `.git`, and `target` beside a `Cargo.toml`. Configuration files
(`toml`, `yaml`, `json`), Dockerfiles and Rust are not source files in another language. There is no allow-list.

`workflow_run` keeps logic out of YAML. It parses each file under `.github/workflows`, and every `action.yml` or `action.yaml`, as
YAML, so how a step is spelled does not matter. The value of a `run` key must be one line of at most
120 characters, with none of the shell operators `|`, `&`, `;`, `>`, `<`, a backtick, `$(` or a `${` expansion with
a modifier, and not start with `if`, `for`, `while`, `until`, `case`, `[`, `[[`, `!` or `(`. It may not run code inline: no `eval`, and no shell or interpreter given a program as an argument (`bash -c`, `python3 -c`, `node -e`, `perl -pe`, `pwsh -Command` and the like). A value that holds several lines fails, however it is written, and so does a value under `env`, so a script cannot wait there for `eval`. The
value of a `shell` key gets the same rules and must start with `bash`, `sh`, `pwsh`, `powershell` or `cmd`. A step that uses
`actions/github-script` fails if it has a `script`. A `uses` that starts with `./` must name a directory with an `action.yml` or `action.yaml`, or a
workflow file in the checked tree; that action is judged like a workflow. A workflow the lint cannot read or parse fails too. There is no allow-list: anything longer is a Rust
command.

The CI helpers are Rust too, and workflows only call them. `ci` has these commands:

    ci timed <measured> <stage> <program> [<argument>...]       time a stage into the measured file
    ci logged <log> <program> [<argument>...]                   run a program, copy its output to the log
    ci libtest-seconds <measured> <stage> <test output>         add up the libtest seconds
    ci reach <metadata> <event> <environment> <summary>         write the scope, say which scopes are empty
    ci tools <built directory> <tools directory>                build (dev profile) and copy reach, lint and ci
    ci mutants-changed <diff> <output> <shard> [<scope>...]     one shard of the mutants on the changed lines, or say it skipped
    ci mutants-shard <shard> <output>                           one weekly shard of all mutants
    ci stage-issues <limits> <measured> [<stage>...]            one issue per stage past its warn level
    ci compile-trend <cold build> <downloads>                   the trend issue when the cold build is slowest
    ci wall-times <repository> <run> <job>                      wall time per step, from the GitHub API
    ci show <file>...                                           print files, and say which are absent
    ci labels <base branch> <pull request>                      the golden-change and new-scenario checks
    ci spec-line <pull request>                                 the spec line of the body, or repair: main
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
A test cannot record a package's golden by passing `Mode::Record` to `verify_in`: into a directory that holds
a `Cargo.toml`, that records only when `GOLDEN_RECORD=1` chose it, and fails otherwise.

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

`mandatory_laws` fails a `pub` type that has a conversion pair, or a hand-written `Ord` or `PartialEq` (a `PartialEq` whose `eq` only compares pointers, such as `Rc::ptr_eq` or `ptr::eq`, needs no law), without a law in a
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

## Spec line

`labels.yml` also runs `ci spec-line`, on every pull request when it is opened, edited, pushed to or reopened. It
fails unless a line of the body is a well-formed spec line, one of:

    spec: none
    spec: owner <uuid prefix of 8 or more hex digits>
    spec: <project>#D<n>
    spec: <project>#L<n>, <project>#D<n>

A project is lowercase letters, digits and hyphens; several labels are separated by commas. The check reads the form
only: it does not look the label up in a spec, since a spec may be local. A pull request that repairs a red main carries
the body line `repair: main` instead, and then passes whatever its spec line says. The output names the case that
applied: repairing main, spec line well formed, spec line malformed (each such line), or spec line missing.

## CI wall-time limits

`ci/limits.toml` holds a warn and a block level in seconds for each timed CI stage: clippy, test compile,
test run and lint. The levels assume a full run, so they apply on main and on pull requests into main; a pull request into
another branch is not judged by them. A stage above its warn level on main opens, or updates, one GitHub issue titled
with the stage name (`ci stage-issues`), so creep becomes a work item before it blocks. The toolchain install has no limit, since it is not our code; it stays in the wall time per step
that main reports. `ci timed` times a stage; the test run is the sum of the `finished in`
times libtest prints (`ci libtest-seconds`), so compile time is not in it. The `limits` step runs `gate-limits <limits> <measured> [<stage>...]`, which prints
each stage as ok, WARN or BLOCK and fails when any stage is above its block level. CI runs `clippy`, `test` and `lint`
as three parallel jobs, and each times and judges only its own stages (`clippy`; `test-compile` and `test-run`; `lint`),
so a stage is still judged alone; a named stage with no limit, or a limit with no measurement, fails. A stage with a limit
but no measurement, or a measurement with no limit, also fails. The levels come from seven cold runs
on `ubuntu-latest`: block just above the slowest run, warn just above the median.
Test compile is judged and printed on every pull request and push but never fails a job (`--report-only test-compile`).
Compile time is tracked by the daily cold build instead: `ci compile-trend` compares its total with each successful cold
build on main in the previous 14 days, downloaded from their artifacts, and when today is slower than every one of them
opens, or updates, one issue titled as a compile-time trend. Its body gives the total against the fortnight's range, the
crates that grew most against their fortnight average, and the pull requests merged into main in the fortnight.
The Rust toolchain, the CI tools (built in the dev profile, so they reuse the test build's compiled crates) and compiled crates (`sccache`) are cached.

## Reach

A pull request pays only for what it can affect. `ci reach` lists the files the pull request changes and
gives them to `reach`, with the output of `cargo metadata --no-deps`:

    reach affected <metadata> [<changed file>...]

A file inside a crate reaches that crate and every crate that depends on it, directly or not, through
normal, dev or build dependencies. `README.md` and `specs/` reach nothing. Any other file reaches every crate:
that covers `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `clippy.toml`, `.cargo/`, `.github/`, `ci/`, the
files of a deleted crate and anything not yet known.
A crate whose tests read files outside its own directory declares it in its manifest, and a change to a file under a
declared path reaches that crate too: `[package.metadata.reach]` with `reads = ["crates"]`. A file `reach` cannot place fails closed.

`affected` prints the share of crates reached and the scopes CI uses, and the run summary of each pull
request shows them. Clippy, the tests and mutants run on that scope, and are skipped when nothing is
reached. A skip is not silent: for each scope that is empty, `ci reach` prints `skipped: <scope> is empty, so the
steps that use it do not run` in the step and in the run summary, and `ci mutants-changed` prints that mutants were
skipped when it gets no scope. The lint step and `cargo fmt` stay whole-repo: lints such as `crate_graph` and `repeated_methods`
cannot judge a slice. Main, and every pull request into main, ignore all this and run everything, with `--workspace`; mutants still test only the changed lines, in the reach. A pull request into any other branch keeps the reach.

## Newtypes

`#[derive(Newtype)]` implements `Wrapping` (`type Inner`, `into_inner`) and `Newtype` (`new`) for a struct with one field,
tuple or named, generic or not. It also writes exactly one `impl newtype::kind::Validity` for the type, with
`type Kind = Unchecked` here and `type Kind = Checked` under `#[newtype(checked)]`, so a type is one or the other.
`Newtype` requires `Validity<Kind = Unchecked>`, so a checked type never gets `new`.

    #[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Rounds(usize);

A validated newtype has no `new`. With `#[newtype(checked)]` the same derive implements `Wrapping` and
`newtype::Guarded` instead. `Guarded::try_new` is the single checked way in: the inherent `try_new`, `CheckedDeserialize`,
the store's reads and `Arbitrary` all go through it. `Guarded` is only ever derived; `hand_impls` fails a hand-written
`impl Guarded`, so no constructor skips the check. The rule lives on the type, in an `impl Checked`: a function from `&Inner`
to `Result<&Inner, Error>`.

    #[derive(Newtype, Arbitrary, Debug)]
    #[newtype(checked)]
    struct Even(u32);

    impl Checked for Even {
        type Error = Odd;

        fn checked(number: &u32) -> Result<&u32, Odd> { ... }
    }

`#[derive(Serialize, Deserialize)]` (`newtype::Serialize`, `newtype::Deserialize`) write and read a newtype as its inner
value; `Deserialize` reads through `Newtype::new`, so only an unchecked newtype can derive it. A validated newtype
derives `CheckedDeserialize` (`newtype::CheckedDeserialize`) instead: it reads through `Guarded::try_new` and is
bounded by `Self: Guarded`, so it does not compile without the check or on a newtype that is not `#[newtype(checked)]`. A value the check refuses is a deserialization
error carrying the check's message, so `Checked::Error` implements `Display`. They reach serde through `newtype::serde`,
so the using crate needs only `newtype`.

`#[derive(Arbitrary)]` (`newtype::Arbitrary`) implements proptest's `Arbitrary` through those constructors, so a validated newtype
only generates values its check admits. It reaches proptest through `newtype::proptest`, so the using crate needs only
`newtype`.

`arbitrary_newtypes` fails a struct in shipped code that derives `Newtype` without `Arbitrary`, so `any::<T>()` works
for every domain value. A newtype that borrows (a reference or a lifetime in its field or generics) passes, since
proptest cannot own what it generates. When the inner type has no `Arbitrary`, the fix is `#[newtype(strategy = ...)]`
or a generator for the inner type: `text_lines::OneLine` and `lint_layout::Name` have hand-written ones.

`#[newtype(strategy = ...)]` gives the derived `Arbitrary` its own proptest strategy for the inner value, for checks
that admit a small share of it (`strategy = 0u32..1000`, `strategy = "[0-9a-f]{40}"`). A validated newtype with a
strategy still goes through its check. The derive also writes an inherent `const fn new` for a plain newtype, so constants can be
built; trait functions cannot yet be `const`. Every other shared helper comes from a blanket trait over `Newtype`,
not from the derive: a zero count is `Counter::zero`. `#[derive(Display)]` forwards `Display` to
the inner value.

`Counter` is a blanket trait over every `Copy` newtype of `usize`: `zero`, `count`, `plus`, `incremented`
and `total`, saturating at the largest count. The orphan rule forbids a blanket `Add` or `Sum`, so these
are named methods. Derived code is not in the source, so `repeated_methods` does not see it, and neither does it
count the impl of a trait (an operator, `Display`, `From`, `AsRef`, `FromStr`). A hand-written constructor or other
public method of one shape on three types still fails, whatever their inner types are (an argument of the inner type
counts as `Inner`); a `const fn` is never counted, since a trait method cannot be const on stable. The fix is `AsRef<T>` for a stored field or `From<&T>` for a computed value, on a newtype, or a derive.

Text is typed: `primitive_boundary` fails a bare `&str` or `String` in a public function signature or field, except in trait impls and inside `ports`; a public trait declaration is checked like a public function, a private one is not, and a type alias is read through (`type Label = String;` then `pub fn f(x: Label)` is flagged, the alias itself is not); use a newtype, with `Display` at the render edge and `FromStr` at the parse edge.

Data read from a file has structure, so `no_plaintext` fails a type that derives `Deserialize` and has a field of `String`, `&str`, `Cow<str>` (also inside `Option`, `Vec` and other generics) or a newtype that wraps one without validation. It reads the code model: a derive is serde's when its path resolves to `Deserialize` in a dependency whose package, as the crate's `Cargo.toml` declares it (renames, `workspace = true` and platform tables included), is `serde` or `serde_derive`; the lint declares those packages as data and never matches the spelling of a path. A newtype around text passes only if it derives `newtype::CheckedDeserialize`, resolved to newtype's own derive. `#[serde(try_from = ...)]`, `transparent` or `from`, an `impl Checked` beside an unchecked derive, another derive named `CheckedDeserialize`, or a hand-written `Deserialize` does not validate. A field type the reader cannot resolve (an external glob may shadow it, so even a plain `String` under `use other::*` is unresolved) is never text: on a type serde deserializes it raises "type not resolvable" at the field, and an explicit import clears it. The reader records why a path is unresolved. When the lookup failed in a module that holds an item-position macro invocation (a `macro_rules!` macro's call, `bitflags!`), and the name is not one a primitive, the prelude, std or a workspace crate supplies, the reader cannot rule out that the macro declared it: on a type serde deserializes, that field raises "type declared by a macro" at the field, and the fix is to declare the type directly, outside the macro. It is never text. A `String` beside an item macro still raises "type not resolvable", since an import settles it. A derive the reader cannot resolve (behind a glob or no import) on a type with a text or unresolvable field raises "derive not resolvable" at the derive; import it explicitly. A derive path into an external crate that no identity row names is followed through that crate's root: the lint asks `cargo metadata --locked --offline` once per workspace for each dependency's lib target, and the reader reads each crate root it needs once per run, caching its top-level `pub use` and `pub extern crate` items, its globs and its declared names (modules and `proc_macro_derive` names) in the model. So `newtype::serde::Deserialize` (newtype's root has `pub use serde;`) and `facade::Deserialize` (a root with `pub use serde::Deserialize;`) are serde's derive. A path through a declared module stays that crate's (`diesel::prelude::Insertable`); a derive's last step prefers a re-export of a proc-macro crate, the way Rust keeps a derive and a trait of one name apart. Only crate roots are read, never their modules. A derive whose crate root cannot be read (no metadata, missing source), whose name the root binds twice in one namespace, hides behind a glob the reader cannot see into, or re-exports in a loop raises "derive not resolvable". CI fetches dependency sources before the lint step for this. A dependency's key is matched to the crate path that names it by its words, so `wire-format` in `Cargo.toml` is the crate `wire_format`. The fix is typed parts (an enum, a number, a `PathBuf`) or a validating newtype such as `text_lines::OneLine`. There is no allow-list.

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
The mutants run in eight `mutants-shard` jobs, each taking every eighth mutant (`--shard k/8 --sharding round-robin`),
so a pull request that changes most of the workspace still fits GitHub's six-hour job limit. The `mutants` job
waits for all of them and passes only if every shard succeeded, so a failed or cancelled shard fails it.

`mutants.yml` runs every mutant once a week, in eight shards, or on request, each against the whole
workspace (`.cargo/mutants.toml`). It reports in the run
summary and the shard artifacts. Missed mutants do not fail it; a shard fails only if the run itself
fails. No push starts it and no push cancels it.

## Fuzzing

The `fuzz` crate feeds the runner arbitrary bytes, arbitrary valid syntax, damaged valid syntax, deep nesting and long chains, and fails when the runner panics or overflows its stack. Its test target needs the `campaign` feature, so `cargo test --workspace` and the PR gates never build or run it; only the `fuzz` workflow does. It uses proptest rather than cargo-fuzz, which needs a nightly toolchain: proptest runs on the pinned stable toolchain, generates valid syntax from strategies, shrinks a failing input, and the targets need no coverage feedback to find a panic in a lint written over `syn`.

    FUZZ_SECONDS=60 FUZZ_CRASHES=/some/dir cargo test -p fuzz --features campaign --test fuzz

Each target runs for `FUZZ_SECONDS` seconds (default 5) with fresh random seeds. With `FUZZ_CRASHES` set, the input being run is first written to `<target>-pending.rs` and `<target>-pending.role`, so an abort that no test can catch still leaves its input behind; a panic also leaves the shrunk input as `<target>-<role>-<hash>.rs`. The `fuzz` workflow runs every day on main for 15 minutes, fails the run on a crash and uploads that directory. A crash found becomes a named test in the crate that owns the fix, with the input kept in its `tests` directory.

Found so far: a method chain of ten thousand links overflowed the stack of the main thread, because `syn` prints and walks a left-nested tree recursively. `Cli::report_to` now does its work on a thread with a 256 MiB stack, which is address space and not memory, and `a_twenty_thousand_link_method_chain_is_linted_without_overflowing_the_stack` holds it.
