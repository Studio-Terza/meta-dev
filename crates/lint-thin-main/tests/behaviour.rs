use lint_thin_main::ThinMain;

use lint_fixtures::{Case, Source, Sources};

const ATTRIBUTED: &str = "#[allow(unused)]\nfn helper() {}\n#[allow(unused)]\nenum E {}\n#[allow(unused)]\nmod m {}\n#[allow(unused)]\nimpl A {}\n#[allow(unused)]\ntrait T {}\n#[allow(unused)]\nconst C: u8 = 1;\n#[allow(unused)]\nstatic S: u8 = 1;\n#[allow(unused)]\ntype Y = u8;\nfn main() {}\n";
const FAT: &str = "use std::fmt;\nfn main() {}\nstruct Extra;\n";

const CASES: [Case; 5] = [
    Case::new(
        "each item on its keyword line",
        &[Source::new("src/main.rs", ATTRIBUTED)],
        &[
            "src/main.rs:2",
            "src/main.rs:4",
            "src/main.rs:6",
            "src/main.rs:8",
            "src/main.rs:10",
            "src/main.rs:12",
            "src/main.rs:14",
            "src/main.rs:16",
        ],
    ),
    Case::new("a library file", &[Source::new("src/lib.rs", FAT)], &[]),
    Case::new("a module named main", &[Source::new("src/tools/main.rs", FAT)], &[]),
    Case::new("a bin directory root", &[Source::new("src/bin/tool/main.rs", FAT)], &["src/bin/tool/main.rs:3"]),
    Case::new("a file beside a bin directory root", &[Source::new("src/bin/tool/helper.rs", FAT)], &[]),
];

#[test]
fn logic_is_reported_at_its_keyword_in_binary_roots_only() {
    assert_eq!(Sources::mismatches::<ThinMain>(&CASES), Vec::<String>::new());
}
