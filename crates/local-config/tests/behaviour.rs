use std::error::Error;
use std::path::{Path, PathBuf};

use design_model::Project;
use file_ports::{Files, Memory};
use laws::proptest::prelude::{any, prop_assert, prop_assert_eq};
use laws::proptest::strategy::Strategy;
use laws::proptest::test_runner::TestCaseError;
use local_config::{ConfigFailure, Kept, LocalConfig};

const REPO: &str = "/repo";

struct Configs;

impl Configs {
    fn any() -> impl Strategy<Value = LocalConfig> {
        (Configs::projects(), any::<String>())
            .prop_map(|(project, workspace)| LocalConfig::new(project, PathBuf::from(workspace)))
    }

    fn projects() -> impl Strategy<Value = Project> {
        "[A-Za-z0-9][A-Za-z0-9._-]{0,15}".prop_filter_map("a project name", |name| name.parse::<Project>().ok())
    }

    fn stored(memory: &Memory) -> Result<LocalConfig, TestCaseError> {
        let text = memory
            .read_text(&Path::new(REPO).join(".meta/local.toml"))
            .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        text.as_ref().parse().map_err(|failure: ConfigFailure| TestCaseError::fail(failure.to_string()))
    }

    fn kept(config: &LocalConfig, memory: &Memory) -> Result<Kept, TestCaseError> {
        config.kept(memory, Path::new(REPO)).map_err(|failure| TestCaseError::fail(failure.to_string()))
    }
}

laws::law!(a_kept_config_reads_back_as_itself, Configs::any(), config, {
    let memory = Memory::default();
    prop_assert_eq!(Configs::kept(&config, &memory)?, Kept::Written);
    prop_assert_eq!(Configs::stored(&memory)?, config);
});

laws::law!(a_kept_config_is_found_at_its_repository_and_nowhere_else, Configs::any(), config, {
    let memory = Memory::default();
    let before =
        LocalConfig::found(&memory, Path::new(REPO)).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(before, None);
    Configs::kept(&config, &memory)?;
    let found =
        LocalConfig::found(&memory, Path::new(REPO)).map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(found, Some(config));
    let elsewhere = LocalConfig::found(&memory, &Path::new(REPO).join("sub"))
        .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
    prop_assert_eq!(elsewhere, None);
});

laws::law!(keeping_the_same_config_twice_changes_nothing, Configs::any(), config, {
    let memory = Memory::default();
    Configs::kept(&config, &memory)?;
    let before = Configs::stored(&memory)?;
    prop_assert_eq!(Configs::kept(&config, &memory)?, Kept::Unchanged);
    prop_assert_eq!(Configs::stored(&memory)?, before);
});

laws::law!(a_repository_kept_for_one_project_refuses_another, (Configs::any(), Configs::projects()), pair, {
    let (config, other) = pair;
    let memory = Memory::default();
    Configs::kept(&config, &memory)?;
    let moved = LocalConfig::new(other, PathBuf::from("/elsewhere"));
    let refused = moved.kept(&memory, Path::new(REPO));
    let elsewhere = matches!(refused, Err(ConfigFailure::Elsewhere { .. }));
    prop_assert!(elsewhere);
    prop_assert_eq!(Configs::stored(&memory)?, config);
});

#[test]
fn a_kept_config_leaves_the_meta_directory_ignored_by_git() -> Result<(), Box<dyn Error>> {
    let memory = Memory::default();
    LocalConfig::new("demo".parse()?, PathBuf::from("/ws")).kept(&memory, Path::new(REPO))?;
    let ignored = memory.read_text(&Path::new(REPO).join(".meta/.gitignore"))?;
    let rules: String = ignored.as_ref().chars().filter(|character| *character != '\n').collect();
    assert_eq!(rules, "*");
    Ok(())
}

laws::roundtrip!(
    a_local_config_is_serialized_and_deserialized_back,
    Configs::any(),
    |config: &LocalConfig| toml::to_string(config).ok(),
    |text: &Option<String>| text.as_deref().and_then(|text| toml::from_str::<LocalConfig>(text).ok()),
    Some
);
