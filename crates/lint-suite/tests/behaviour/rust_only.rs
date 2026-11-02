use std::fs;

use lint_core::RepoFile;
use lint_rust_only::RustOnly;

const EXTENSIONS: [(&str, &str); 83] = [
    ("py", "Python"),
    ("pyw", "Python"),
    ("pyi", "Python"),
    ("sh", "shell"),
    ("bash", "shell"),
    ("zsh", "shell"),
    ("fish", "shell"),
    ("ksh", "shell"),
    ("csh", "shell"),
    ("bat", "batch"),
    ("cmd", "batch"),
    ("ps1", "PowerShell"),
    ("psm1", "PowerShell"),
    ("pl", "Perl"),
    ("pm", "Perl"),
    ("rb", "Ruby"),
    ("php", "PHP"),
    ("lua", "Lua"),
    ("go", "Go"),
    ("java", "Java"),
    ("kt", "Kotlin"),
    ("kts", "Kotlin"),
    ("scala", "Scala"),
    ("swift", "Swift"),
    ("m", "Objective-C"),
    ("mm", "Objective-C"),
    ("c", "C"),
    ("h", "C"),
    ("cc", "C++"),
    ("cpp", "C++"),
    ("cxx", "C++"),
    ("hh", "C++"),
    ("hpp", "C++"),
    ("hxx", "C++"),
    ("cs", "C#"),
    ("dart", "Dart"),
    ("hs", "Haskell"),
    ("ex", "Elixir"),
    ("exs", "Elixir"),
    ("erl", "Erlang"),
    ("clj", "Clojure"),
    ("cljs", "Clojure"),
    ("ml", "OCaml"),
    ("mli", "OCaml"),
    ("fs", "F#"),
    ("fsx", "F#"),
    ("r", "R"),
    ("jl", "Julia"),
    ("groovy", "Groovy"),
    ("zig", "Zig"),
    ("dash", "shell"),
    ("t", "Perl"),
    ("rake", "Ruby"),
    ("erb", "Ruby"),
    ("cljc", "Clojure"),
    ("pyx", "Python"),
    ("hrl", "Erlang"),
    ("awk", "awk"),
    ("sed", "sed"),
    ("tcl", "Tcl"),
    ("nix", "Nix"),
    ("mk", "make"),
    ("ipynb", "notebook"),
    ("vim", "Vim script"),
    ("el", "Lisp"),
    ("lisp", "Lisp"),
    ("scm", "Scheme"),
    ("rkt", "Racket"),
    ("nim", "Nim"),
    ("vb", "Visual Basic"),
    ("vbs", "Visual Basic"),
    ("sol", "Solidity"),
    ("asm", "assembly"),
    ("s", "assembly"),
    ("d", "D"),
    ("pas", "Pascal"),
    ("f90", "Fortran"),
    ("cr", "Crystal"),
    ("v", "V"),
    ("ino", "Arduino"),
    ("bzl", "Starlark"),
    ("PY", "Python"),
    ("Sh", "shell"),
];

const WEB: [(&str, &str); 15] = [
    ("js", "JavaScript"),
    ("mjs", "JavaScript"),
    ("cjs", "JavaScript"),
    ("jsx", "JavaScript"),
    ("ts", "TypeScript"),
    ("tsx", "TypeScript"),
    ("mts", "TypeScript"),
    ("cts", "TypeScript"),
    ("html", "HTML"),
    ("htm", "HTML"),
    ("css", "CSS"),
    ("scss", "CSS"),
    ("vue", "Vue"),
    ("svelte", "Svelte"),
    ("astro", "Astro"),
];

const REWRITE: &str = "rewrite it in Rust";
const MOVE: &str = "if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust";

struct Checked;

impl Checked {
    fn report(files: &[RepoFile]) -> Vec<String> {
        RustOnly::report(files).violations().iter().map(ToString::to_string).collect()
    }

    fn one(path: &str) -> Vec<String> {
        Self::report(&[RepoFile::new(path, path)])
    }

    fn within(checkout: &str, path: &str) -> Vec<String> {
        Self::report(&[RepoFile::new(format!("{checkout}/{path}"), path)])
    }
}

#[test]
fn every_listed_extension_names_its_language_and_the_fix() {
    for (extension, name) in EXTENSIONS {
        assert_eq!(
            Checked::one(&format!("tools/run.{extension}")),
            [format!("tools/run.{extension}:1: {name} source outside assets; {REWRITE}")]
        );
    }
}

#[test]
fn web_languages_are_told_to_move_to_assets() {
    for (extension, name) in WEB {
        assert_eq!(
            Checked::one(&format!("site/app.{extension}")),
            [format!("site/app.{extension}:1: {name} source outside assets; {MOVE}")]
        );
    }
}

#[test]
fn rust_and_configuration_files_pass() {
    for path in [
        "src/lib.rs",
        "Cargo.toml",
        "ci/limits.toml",
        ".github/workflows/ci.yml",
        "Dockerfile",
        "README.md",
        "data.json",
        "noextension",
        ".hidden",
        "scripts.py/data.txt",
    ] {
        assert_eq!(Checked::one(path), Vec::<String>::new(), "{path}");
    }
}

#[test]
fn a_web_file_below_a_directory_named_assets_passes() {
    for path in ["assets/app.js", "site/assets/app.js", "site/assets/deep/er/app.css", "assets/page.html"] {
        assert_eq!(Checked::one(path), Vec::<String>::new(), "{path}");
    }
}

#[test]
fn a_script_below_a_directory_named_assets_still_fails() {
    assert_eq!(
        Checked::one("site/assets/deep/run.py"),
        [format!("site/assets/deep/run.py:1: Python source below assets is not a web asset; {REWRITE}")]
    );
    assert_eq!(
        Checked::one("assets/x.sh"),
        [format!("assets/x.sh:1: shell source below assets is not a web asset; {REWRITE}")]
    );
}

#[test]
fn only_a_directory_named_exactly_assets_counts() {
    for path in [
        "my-assets/app.js",
        "assets-old/app.js",
        "assets.js",
        "static/assets_app.js",
        "x/not_assets/run.sh",
        "assets.d/run.sh",
    ] {
        assert_eq!(Checked::one(path).len(), 1, "{path}");
    }
}

#[test]
fn the_directories_above_the_checkout_do_not_count() {
    assert_eq!(
        Checked::within("/home/assets/repo", "run.sh"),
        ["/home/assets/repo/run.sh:1: shell source outside assets; rewrite it in Rust"]
    );
    assert_eq!(Checked::within("/home/repo", "assets/app.js"), Vec::<String>::new());
}

#[test]
fn findings_come_in_the_order_given() {
    let files = ["b.sh", "a.py", "c.rs"].map(|path| RepoFile::new(path, path));
    assert_eq!(
        Checked::report(&files),
        [
            format!("b.sh:1: shell source outside assets; {REWRITE}"),
            format!("a.py:1: Python source outside assets; {REWRITE}"),
        ]
    );
}

#[test]
fn a_report_with_no_findings_is_clean() {
    assert!(RustOnly::report(&[]).is_clean());
    assert!(!RustOnly::report(&[RepoFile::new("a.sh", "a.sh")]).is_clean());
}

#[test]
fn an_extensionless_file_with_a_foreign_shebang_fails() {
    for first in [
        "#!/bin/bash",
        "#!/usr/bin/env python3",
        "#!/usr/bin/env -S node --flag",
        "#!/usr/bin/env FOO=1 perl",
        "#! /bin/sh -e",
        "#!",
        "#!/usr/bin/env",
    ] {
        let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
        fs::write(checkout.path().join("deploy"), format!("{first}\nfor i in a b; do echo $i; done\n")).ok();
        assert_eq!(
            Checked::within(&checkout.path().display().to_string(), "deploy"),
            [format!("{}/deploy:1: interpreted script source outside assets; {REWRITE}", checkout.path().display())],
            "{first}"
        );
    }
}

#[test]
fn an_extensionless_file_without_a_foreign_shebang_passes() {
    for text in [
        "#!/usr/bin/env rust-script\nfn main() {}\n",
        "#!/usr/bin/env -S cargo +nightly -Zscript\nfn main() {}\n",
        "#!/home/me/.cargo/bin/cargo\n",
        "# not a shebang\n",
        "echo hi\n",
        "",
        " #!/bin/sh\n",
        "\n#!/bin/sh\n",
    ] {
        let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
        fs::write(checkout.path().join("tool"), text).ok();
        assert_eq!(Checked::within(&checkout.path().display().to_string(), "tool"), Vec::<String>::new(), "{text:?}");
    }
}

#[test]
fn a_foreign_shebang_in_a_file_with_a_data_extension_fails_but_a_rust_attribute_passes() {
    let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
    let root = checkout.path().display().to_string();
    fs::write(checkout.path().join("notes.txt"), "#!/bin/bash\n").ok();
    fs::write(checkout.path().join("lib.rs"), "#!/usr/bin/env python3\n").ok();
    fs::write(checkout.path().join("inner.txt"), "#![allow(dead_code)]\n").ok();
    assert_eq!(
        Checked::within(&root, "notes.txt"),
        [format!("{root}/notes.txt:1: interpreted script source outside assets; {REWRITE}")]
    );
    assert_eq!(Checked::within(&root, "lib.rs"), Vec::<String>::new());
    assert_eq!(Checked::within(&root, "inner.txt"), Vec::<String>::new());
}

#[test]
fn a_shebang_below_a_directory_named_assets_still_fails() {
    let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
    fs::create_dir_all(checkout.path().join("assets")).ok();
    fs::write(checkout.path().join("assets/run"), "#!/usr/bin/env node\n").ok();
    assert_eq!(Checked::within(&checkout.path().display().to_string(), "assets/run").len(), 1);
}

#[test]
fn vendored_code_of_any_language_is_not_the_repositorys_logic() {
    let vendored: Vec<RepoFile> = ["vendor/run.py", "third/vendor/deep/app.js", "vendor/x.sh", "site/vendor/app.css"]
        .into_iter()
        .map(|path| RepoFile::new(path, path))
        .collect();
    assert_eq!(Checked::report(&vendored), Vec::<String>::new());
}

#[test]
fn a_file_merely_named_vendor_still_fails() {
    assert_eq!(
        Checked::one("tools/vendor.py"),
        [format!("tools/vendor.py:1: Python source outside assets; {REWRITE}")]
    );
}

#[test]
fn a_source_extension_hidden_behind_another_extension_is_still_source() {
    for (path, name) in [
        ("ios/Tests.swift.txt", "Swift"),
        ("tools/run.py.bak", "Python"),
        ("tools/run.SH.txt", "shell"),
        ("tools/run.go.orig", "Go"),
        ("tools/run.js.disabled", "JavaScript"),
    ] {
        let fix = if name == "JavaScript" { MOVE } else { REWRITE };
        assert_eq!(Checked::one(path), [format!("{path}:1: {name} source outside assets; {fix}")], "{path}");
    }
}

#[test]
fn a_double_extension_that_hides_no_source_passes() {
    for path in ["data.tar.gz", "notes.txt.bak", "report.final.txt", "scripts.py/data.txt", "a.rs.txt", "x.min.json"] {
        assert_eq!(Checked::one(path), Vec::<String>::new(), "{path}");
    }
}

#[test]
fn test_code_of_any_language_passes_below_platform() {
    for path in [
        "platform/tests/Run.swift",
        "ios/platform/UITests/Flow.swift",
        "platform/ios/BashoUITests/Flow.swift",
        "platform/ios/FlowTests.swift",
        "platform/android/test/Flow.kt",
        "platform/tests/helper.py",
    ] {
        assert_eq!(Checked::one(path), Vec::<String>::new(), "{path}");
    }
}

#[test]
fn code_below_platform_that_is_not_test_code_fails() {
    for (path, name) in
        [("platform/ios/App.swift", "Swift"), ("x/platform/run.sh", "shell"), ("platform/Testing.py", "Python")]
    {
        assert_eq!(
            Checked::one(path),
            [format!("{path}:1: {name} source below platform is not test code; {REWRITE}")],
            "{path}"
        );
    }
}

#[test]
fn only_a_directory_named_exactly_platform_counts() {
    for path in ["platforms/tests/Run.swift", "my-platform/tests/run.sh", "platform.d/tests/run.py", "tests/run.py"] {
        assert_eq!(Checked::one(path).len(), 1, "{path}");
    }
}

#[test]
fn a_file_whose_content_is_source_fails_whatever_its_name() {
    for (name, text, language) in [
        ("Flow.txt", "import XCTest\n\nfinal class Flow: XCTestCase {}\n", "Swift"),
        ("ui", "import SwiftUI\n", "Swift"),
        ("tool", "import os\nimport sys\n", "Python"),
        ("tool.txt", "def main():\n    pass\n", "Python"),
        ("tool.dat", "from pathlib import Path\nimport os\n", "Python"),
        ("deploy", "set -euo pipefail\n", "shell"),
        ("deploy.txt", "if [ -f x ]; then\n  echo hi\nfi\n", "shell"),
        ("deploy.cfg", "for f in a b; do\n  echo $f\ndone\n", "shell"),
        ("main.txt", "#include <stdio.h>\n", "C"),
    ] {
        let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
        let root = checkout.path().display().to_string();
        fs::write(checkout.path().join(name), text).ok();
        assert_eq!(
            Checked::within(&root, name),
            [format!("{root}/{name}:1: {language} source outside assets; {REWRITE}")],
            "{name}"
        );
    }
}

#[test]
fn ordinary_text_and_data_pass() {
    for (name, text) in [
        ("notes.txt", "We must import the data from the old system.\nThen we are done.\n"),
        ("notes.md", "# Title\n\n```python\nimport os\ndef main():\n    pass\n```\n\n```sh\nset -e\n```\n"),
        ("data.json", "{\"def\": \"x(y):\"}\n"),
        ("config.toml", "[package]\nname = \"x\"\n"),
        ("flow.yml", "run: |\n  import os\n  def f():\n"),
        ("log.txt", "import duty\nfi\n"),
        ("blob.bin", "import os\u{0}import sys\n"),
        ("empty.txt", ""),
        ("poem", "Roses are red\nimport this\n"),
    ] {
        let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
        fs::write(checkout.path().join(name), text).ok();
        assert_eq!(Checked::within(&checkout.path().display().to_string(), name), Vec::<String>::new(), "{name}");
    }
}

#[test]
fn source_by_content_below_platform_tests_passes() {
    let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
    fs::create_dir_all(checkout.path().join("platform/UITests")).ok();
    fs::write(checkout.path().join("platform/UITests/Flow.txt"), "import XCTest\n").ok();
    assert_eq!(
        Checked::within(&checkout.path().display().to_string(), "platform/UITests/Flow.txt"),
        Vec::<String>::new()
    );
}
