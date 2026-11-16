use lint_core::{Lint, SourceFile};
use lint_file::{Filed, OneWalk};
use lint_layout::SourceText;
use lint_sweep::Sweep;

#[test]
fn a_file_walked_again_in_the_same_run_is_flagged() -> Result<(), lint_layout::Failure> {
    let once = SourceFile::parse("a.rs", SourceText::new("pub struct A;\n".to_owned()))?;
    let twice = SourceFile::parse("b.rs", SourceText::new("pub struct B;\n".to_owned()))?;
    Sweep::watching(&twice, Vec::new());
    Sweep::watching(&twice, Vec::new());
    let files = [once, twice];
    let report = Filed::<OneWalk>::report(&files);
    let flagged: Vec<String> = report.violations().iter().map(ToString::to_string).collect();
    assert_eq!(
        flagged,
        [
            "b.rs:1: file walked more than once in one run; ride the one sweep with a watcher, a scoped watcher or a lowered fact"
        ]
    );
    Ok(())
}
