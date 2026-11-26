#[path = "generated/laws.rs"]
mod generated_laws;

use std::error::Error;
use workspace_metadata::CrateName;

use workspace_metadata::{Graph, Library, Metadata};

#[test]
fn real_cargo_metadata_lists_each_dependency_library_with_its_root_form_and_renamed_normal_dependencies()
-> Result<(), Box<dyn Error>> {
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
    golden::golden!("libraries.txt", &format!("{}\n", listed.join("\n")));
    Ok(())
}

macro_rules! ported {
    ($table:literal) => {
        concat!(
            "{\"workspace_root\": \"/w\", \"packages\": [{\"name\": \"store\", \"manifest_path\": \"/w/crates/store/Cargo.toml\", \"dependencies\": [], \"metadata\": {\"port\": ",
            $table,
            "}}]}"
        )
    };
}

laws::table!(
    a_port_table_reads_only_the_keys_it_knows,
    |text: &&str| serde_json::from_str::<Metadata>(text).is_ok(),
    [
        ported!("{\"owns\": [\"rusqlite\"]}") => true,
        ported!("{\"own\": [\"rusqlite\"]}") => false,
        ported!("{\"owns\": [], \"expander\": {\"role\": \"store\", \"macros\": []}}") => false,
    ]
);

macro_rules! member {
    ($name:literal) => {
        concat!(
            "{\"name\": \"",
            $name,
            "\", \"id\": \"",
            $name,
            "\", \"manifest_path\": \"/w/",
            $name,
            "/Cargo.toml\", \"dependencies\": []}"
        )
    };
}

macro_rules! needs {
    ($name:literal, $kind:literal) => {
        concat!(
            "{\"name\": \"",
            $name,
            "\", \"pkg\": \"",
            $name,
            "\", \"dep_kinds\": [{\"kind\": ",
            $kind,
            ", \"target\": null}]}"
        )
    };
}

const CHAINED: &str = concat!(
    "{\"workspace_root\": \"/w\", \"workspace_members\": [\"app\", \"core\", \"base\", \"tools\"], \"packages\": [",
    member!("app"),
    ", ",
    member!("core"),
    ", ",
    member!("base"),
    ", ",
    member!("tools"),
    "], \"resolve\": {\"nodes\": [",
    "{\"id\": \"app\", \"dependencies\": [\"core\", \"tools\"], \"deps\": [",
    needs!("core", "null"),
    ", ",
    needs!("tools", "\"dev\""),
    "]}, {\"id\": \"core\", \"dependencies\": [\"base\"], \"deps\": [",
    needs!("base", "null"),
    "]}, {\"id\": \"base\", \"dependencies\": [], \"deps\": []}, {\"id\": \"tools\", \"dependencies\": [], \"deps\": []}]}}"
);

#[test]
fn each_member_knows_its_depth_and_the_workspace_crates_below_and_above_it_on_the_compile_chain()
-> Result<(), Box<dyn Error>> {
    let metadata: Metadata = serde_json::from_str(CHAINED)?;
    let graph = Graph::from(&metadata);
    let placed: Vec<String> =
        graph.chained().iter().map(|chained| format!("{}: {chained}", AsRef::<CrateName>::as_ref(chained))).collect();
    assert_eq!(
        placed,
        [
            "app: depth 3 on the compile chain, 2 workspace crates below it, 0 above it",
            "base: depth 1 on the compile chain, 0 workspace crates below it, 2 above it",
            "core: depth 2 on the compile chain, 1 workspace crates below it, 1 above it",
            "tools: depth 1 on the compile chain, 0 workspace crates below it, 0 above it",
        ]
    );
    Ok(())
}

#[test]
fn a_lexer_dependency_reaching_an_owned_package_breaches_unless_a_proc_macro_stands_between()
-> Result<(), Box<dyn Error>> {
    let unleafed = |fixture: &str| -> Result<Vec<String>, serde_json::Error> {
        let metadata: Metadata = serde_json::from_str(fixture)?;
        let graph = Graph::from(&metadata);
        Ok(graph.role_breaches().unleafed().iter().map(ToString::to_string).collect())
    };
    assert_eq!(unleafed(include_str!("fixtures/lexer-through-library.json"))?.len(), 1);
    assert_eq!(unleafed(include_str!("fixtures/lexer-through-macro.json"))?, Vec::<String>::new());
    Ok(())
}

#[test]
fn a_crate_reaching_an_owned_package_only_through_a_proc_macro_breaches_no_role() -> Result<(), Box<dyn Error>> {
    let metadata: Metadata = serde_json::from_str(include_str!("fixtures/roles.json"))?;
    let graph = Graph::from(&metadata);
    let breaches = graph.role_breaches();
    let breaching: Vec<String> = breaches
        .unported()
        .iter()
        .map(ToString::to_string)
        .chain(breaches.unroled().iter().map(ToString::to_string))
        .chain(breaches.unleafed().iter().map(ToString::to_string))
        .chain(breaches.reformatted().iter().map(ToString::to_string))
        .collect();
    golden::golden!("roles.txt", &format!("{}\n", breaching.join("\n")));
    Ok(())
}
