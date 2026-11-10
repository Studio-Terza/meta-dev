#![deny(clippy::wildcard_enum_match_arm)]

use grammar_branch::Base;

laws::table!(
    a_base_branch_is_main_a_project_branch_or_another,
    |name: &&str| Base::from(*name),
    [
        "main" => Base::Main,
        "project/transcript-design" => Base::Project,
        "project/x/y" => Base::Project,
        "project/" => Base::Other,
        "project" => Base::Other,
        "projects/x" => Base::Other,
        "feature/project/x" => Base::Other,
        "maintenance" => Base::Other,
        "main/x" => Base::Other,
        "" => Base::Other,
    ]
);
