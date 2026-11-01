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
repeated_methods, state_graph.

Every lint implements `Lint` and takes the same files. Limits are constants inside each lint, not
settings. Files under `tests`, `examples` and `benches` are checked only by the lints that are not
about shipped code; code under `#[cfg(test)]` is skipped as before.

Crates: `lint-core` (reports, sites, violations, source files), `lints` (the lints and runner),
`lint` (the binary), `golden` (golden files for tests).

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
