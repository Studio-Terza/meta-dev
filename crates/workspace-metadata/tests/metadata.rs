use test_failure::Failed;
use workspace_metadata::{CrateName, Library, Metadata};

#[test]
fn real_cargo_metadata_lists_each_dependency_library_with_its_root_form_and_renamed_normal_dependencies()
-> Result<(), Failed> {
    let metadata: Metadata = serde_json::from_str(include_str!("fixtures/metadata.json"))?;
    let listed: Vec<String> = metadata
        .libraries()
        .iter()
        .flat_map(|library: &Library<'_>| {
            let named =
                format!("{} {:?} {}", AsRef::<CrateName>::as_ref(library), library.form(), library.root().display());
            let dependencies = library.dependencies().iter().map(|(krate, package)| format!("  {krate} is {package}"));
            std::iter::once(named).chain(dependencies)
        })
        .collect();
    assert_eq!(
        listed,
        [
            "facade Library /ws/vendor/facade/src/lib.rs",
            "  text is serde",
            "assertions Library /ws/vendor/assertions/src/lib.rs",
            "serde Library /ws/vendor/serde/src/lib.rs",
            "  serde_derive is serde_derive",
            "serde_derive ProcMacro /ws/vendor/serde_derive/src/lib.rs",
        ]
    );
    Ok(())
}
