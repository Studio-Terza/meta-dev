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
        "x.py.txt",
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
