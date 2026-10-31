# meta-dev

Source lints for Rust repositories, run through one binary.

    cargo run -p lint -- <path>

The runner reads every `.rs` file under the path (skipping `target` and hidden directories),
runs all lints, prints the violations, and exits non-zero if there are any.

Lints: no_comments, no_free_fns, thin_main, vec_out_params, named_lifetimes, enum_predicates,
const_shape, type_shape, distinct_signatures, expression_flow, fn_shape, literal_names,
manual_iteration, module_nesting, named_numbers, no_forwarders, no_numbers_in_binaries,
no_parameter_bags, primitive_boundary, private_fns, process_starters, public_surface,
repeated_methods, state_graph.

Every lint implements `Lint` and takes the same files. Limits are constants inside each lint, not
settings. Files under `tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

Crates: `lint-core` (reports, sites, violations, source files), `lints` (the lints and runner),
`lint` (the binary).
