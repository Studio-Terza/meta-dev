use lint_thin_main::ThinMain;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const ATTRIBUTED: &str = "#[allow(unused)]\nfn helper() {}\n#[allow(unused)]\nenum E {}\n#[allow(unused)]\nmod m {}\n#[allow(unused)]\nimpl A {}\n#[allow(unused)]\ntrait T {}\n#[allow(unused)]\nconst C: u8 = 1;\n#[allow(unused)]\nstatic S: u8 = 1;\n#[allow(unused)]\ntype Y = u8;\nfn main() {}\n";
const FAT: &str = "use std::fmt;\nfn main() {}\nstruct Extra;\n";

const CASES: [Case; 5] = [
    Case::new(
        Title::new("each item on its keyword line"),
        &[Source::new(Located::new("src/main.rs"), Text::new(ATTRIBUTED))],
        Flagged::new(&[
            "src/main.rs:2",
            "src/main.rs:4",
            "src/main.rs:6",
            "src/main.rs:8",
            "src/main.rs:10",
            "src/main.rs:12",
            "src/main.rs:14",
            "src/main.rs:16",
        ]),
    ),
    Case::new(
        Title::new("a library file"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(FAT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a module named main"),
        &[Source::new(Located::new("src/tools/main.rs"), Text::new(FAT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a bin directory root"),
        &[Source::new(Located::new("src/bin/tool/main.rs"), Text::new(FAT))],
        Flagged::new(&["src/bin/tool/main.rs:3"]),
    ),
    Case::new(
        Title::new("a file beside a bin directory root"),
        &[Source::new(Located::new("src/bin/tool/helper.rs"), Text::new(FAT))],
        Flagged::new(&[]),
    ),
];

#[test]
fn logic_is_reported_at_its_keyword_in_binary_roots_only() {
    assert_eq!(Sources::mismatches::<ThinMain>(&CASES), Lines::default());
}
