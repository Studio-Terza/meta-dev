use std::path::Path;

#[derive(Debug)]
pub(crate) struct BaselineNames;

impl BaselineNames {
    const MARKS: [&'static str; 11] = [
        "baseline",
        "Baseline",
        "BASELINE",
        "allowlist",
        "Allowlist",
        "AllowList",
        "allow-list",
        "allow_list",
        "ignorelist",
        "IgnoreList",
        "ignore-list",
    ];
    const CODE: &'static str = "rs";
    const CORPUS: &'static str = "fixtures";

    pub(crate) fn listed(path: &Path) -> bool {
        path.extension().is_none_or(|extension| extension != Self::CODE)
            && !path.components().any(|part| part.as_os_str() == Self::CORPUS)
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| Self::MARKS.iter().any(|mark| name.contains(mark)))
    }
}
