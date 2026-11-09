use std::error::Error;

use workspace_metadata::{Library, Metadata};

#[test]
fn real_cargo_metadata_lists_each_dependency_library_with_its_root_form_and_renamed_normal_dependencies()
-> Result<(), Box<dyn Error>> {
    let metadata: Metadata = serde_json::from_str(include_str!("fixtures/metadata.json"))?;
    let listed: Vec<String> = metadata
        .libraries()
        .iter()
        .flat_map(|library: &Library<'_>| {
            let named = format!("{} {:?} {}", library.name(), library.form(), library.root().display());
            let dependencies = library.dependencies().iter().map(|(krate, package)| format!("  {krate} is {package}"));
            std::iter::once(named).chain(dependencies)
        })
        .collect();
    golden::golden!("libraries.txt", &format!("{}\n", listed.join("\n")));
    Ok(())
}
