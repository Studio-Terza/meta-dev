use std::path::Path;

#[derive(Debug)]
pub(crate) struct BaselineNames;

impl BaselineNames {
    const MARKS: [&'static str; 13] = [
        "baseline",
        "Baseline",
        "BASELINE",
        "allowlist",
        "Allowlist",
        "AllowList",
        "allow-list",
        "allow_list",
        "suppressions",
        "Suppressions",
        "ignorelist",
        "IgnoreList",
        "ignore-list",
    ];
    const CODE: &'static str = "rs";

    pub(crate) fn listed(path: &Path) -> bool {
        path.extension().is_none_or(|extension| extension != Self::CODE)
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| Self::MARKS.iter().any(|mark| name.contains(mark)))
    }
}
