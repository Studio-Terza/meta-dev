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
repeated_methods, state_graph, crate_graph.

Every lint implements `Lint` and takes the same files. Limits are constants inside each lint, not
settings. Files under `tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

Crates: `lint-core` (the `Lint` trait, reports, sites, violations, source files), one `lint-<name>` crate
per lint (`lint-crate-graph` among them), `lint-fixtures` (shared test sources), `lint-runner` (the runner
library) and `lint` (the binary). Crates stay split by lint because `crate_graph` flags a crate whose items
fall into two loosely linked groups of five or more, or whose cycle spans more than 40 items; its conductance
floor is 19%.
