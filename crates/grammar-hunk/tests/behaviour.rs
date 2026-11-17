#![deny(clippy::wildcard_enum_match_arm)]

use std::path::{Path, PathBuf};

use grammar_hunk::{Changed, Patch};

const TWO_FILES: &str = "diff --git a/src/a.rs b/src/a.rs\nindex 1..2 100644\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -3,0 +4,2 @@ fn f() {\n+    one();\n+    two();\n@@ -10,2 +12 @@\n--- looks like a header\n-+++ b/looks/like/a/header\n+    three();\ndiff --git a/src/b.rs b/src/b.rs\nnew file mode 100644\nindex 0..3\n--- /dev/null\n+++ b/src/b.rs\n@@ -0,0 +1 @@\n+fn b() {}\n\\ No newline at end of file\n";
const DELETED: &str = "diff --git a/tests/t.rs b/tests/t.rs\ndeleted file mode 100644\nindex 3..0\n--- a/tests/t.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-#[test]\n-fn t() {}\n";
const SHORT: &str = "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,2 +1,2 @@\n-one\n";
const STRAY: &str = "-a line outside any hunk\n";

#[derive(Debug, PartialEq, Eq)]
struct Read {
    before: Option<String>,
    removing: Option<PathBuf>,
}

impl Read {
    fn all(text: &&str) -> Option<Vec<Read>> {
        text.parse::<Patch>().ok().map(|patch| patch.changed().iter().map(Read::of).collect())
    }

    fn of(changed: &Changed) -> Read {
        Read { before: changed.before().map(str::to_owned), removing: changed.removing().map(Path::to_path_buf) }
    }

    fn file(before: Option<&str>, removing: Option<&str>) -> Read {
        Read { before: before.map(str::to_owned), removing: removing.map(PathBuf::from) }
    }
}

struct Old(u32);

impl Old {
    fn moved(&self) -> Option<u32> {
        TWO_FILES.parse::<Patch>().ok()?.changed().first()?.moved(self.0)
    }
}

laws::table!(
    a_zero_context_patch_reads_as_each_file_s_paths_and_hunks_counting_content_lines_whatever_they_look_like,
    Read::all,
    [
        TWO_FILES => Some(vec![
            Read::file(Some("src/a.rs"), Some("src/a.rs")),
            Read::file(None, None),
        ]),
        DELETED => Some(vec![Read::file(Some("tests/t.rs"), Some("tests/t.rs"))]),
        "" => Some(Vec::new()),
        SHORT => None,
        STRAY => None,
    ]
);

laws::table!(
    an_old_line_moves_by_the_hunks_before_it_and_a_removed_line_has_no_new_place,
    Old::moved,
    [
        Old(1) => Some(1),
        Old(3) => Some(3),
        Old(4) => Some(6),
        Old(9) => Some(11),
        Old(10) => None,
        Old(11) => None,
        Old(12) => Some(13),
    ]
);
