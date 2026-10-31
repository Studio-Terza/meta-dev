# meta-dev

Source lints for Rust repositories, run through one binary.

    cargo run -p lint -- <path>

The runner reads every `.rs` file under the path (skipping `target` and hidden directories),
runs all lints, prints the violations, and exits non-zero if there are any.

Lints: no_comments, no_free_fns, thin_main, vec_out_params, named_lifetimes, enum_predicates,
const_shape, type_shape.

Crates: `lint-core` (reports, sites, violations, source files), `lints` (the lints and runner),
`lint` (the binary).
