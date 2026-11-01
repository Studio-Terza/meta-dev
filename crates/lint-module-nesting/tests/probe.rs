use lint_module_nesting::ModuleNesting;

#[test]
fn probe_name_is_the_lint_name() {
    assert_eq!(ModuleNesting::probe_name(), "module_nesting");
}
