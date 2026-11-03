mod content;
mod extensions;
mod fix;

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use content::Content;
use extensions::Extensions;
pub(crate) use fix::Fix;
use lint_core::RepoFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Python,
    Shell,
    Batch,
    PowerShell,
    Perl,
    Ruby,
    Php,
    Lua,
    Go,
    Java,
    Kotlin,
    Scala,
    Swift,
    ObjectiveC,
    C,
    Cpp,
    CSharp,
    Dart,
    Haskell,
    Elixir,
    Erlang,
    Clojure,
    OCaml,
    FSharp,
    R,
    Julia,
    Groovy,
    Zig,
    JavaScript,
    TypeScript,
    Html,
    Css,
    Vue,
    Svelte,
    Awk,
    Sed,
    Tcl,
    Nix,
    Make,
    Notebook,
    Vim,
    Lisp,
    Scheme,
    Racket,
    Nim,
    VisualBasic,
    Solidity,
    Assembly,
    D,
    Pascal,
    Fortran,
    Crystal,
    V,
    Arduino,
    Starlark,
    Astro,
    Script,
}

impl Language {
    const ALL: &'static [Language] = &[
        Language::Python,
        Language::Shell,
        Language::Batch,
        Language::PowerShell,
        Language::Perl,
        Language::Ruby,
        Language::Php,
        Language::Lua,
        Language::Go,
        Language::Java,
        Language::Kotlin,
        Language::Scala,
        Language::Swift,
        Language::ObjectiveC,
        Language::C,
        Language::Cpp,
        Language::CSharp,
        Language::Dart,
        Language::Haskell,
        Language::Elixir,
        Language::Erlang,
        Language::Clojure,
        Language::OCaml,
        Language::FSharp,
        Language::R,
        Language::Julia,
        Language::Groovy,
        Language::Zig,
        Language::JavaScript,
        Language::TypeScript,
        Language::Html,
        Language::Css,
        Language::Vue,
        Language::Svelte,
        Language::Awk,
        Language::Sed,
        Language::Tcl,
        Language::Nix,
        Language::Make,
        Language::Notebook,
        Language::Vim,
        Language::Lisp,
        Language::Scheme,
        Language::Racket,
        Language::Nim,
        Language::VisualBasic,
        Language::Solidity,
        Language::Assembly,
        Language::D,
        Language::Pascal,
        Language::Fortran,
        Language::Crystal,
        Language::V,
        Language::Arduino,
        Language::Starlark,
        Language::Astro,
        Language::Script,
    ];

    const RUST: &'static str = "rs";

    #[must_use]
    pub fn detect(file: &RepoFile) -> Option<Language> {
        Some(file.shown())
            .filter(|path| path.extension().is_none_or(|extension| extension.to_ascii_lowercase() != *Self::RUST))
            .and_then(|path| {
                path.extension()
                    .and_then(Self::by_extension)
                    .or_else(|| path.file_stem().map(Path::new).and_then(Path::extension).and_then(Self::by_extension))
                    .or_else(|| Content::language_of(file.contents()))
            })
    }

    fn by_extension(extension: &OsStr) -> Option<Language> {
        Self::ALL.iter().copied().find(|language| language.extensions().holding(extension).is_some())
    }

    #[must_use]
    pub fn fix(self) -> Fix {
        match self {
            Language::JavaScript
            | Language::TypeScript
            | Language::Html
            | Language::Css
            | Language::Vue
            | Language::Svelte
            | Language::Astro => Fix::MoveToAssets,
            _ => Fix::Rewrite,
        }
    }

    fn extensions(self) -> Extensions {
        match self {
            Language::Python => Extensions::new(&["py", "pyw", "pyi", "pyx"]),
            Language::Shell => Extensions::new(&["sh", "bash", "zsh", "fish", "ksh", "csh", "dash"]),
            Language::Batch => Extensions::new(&["bat", "cmd"]),
            Language::PowerShell => Extensions::new(&["ps1", "psm1"]),
            Language::Perl => Extensions::new(&["pl", "pm", "t"]),
            Language::Ruby => Extensions::new(&["rb", "rake", "erb"]),
            Language::Php => Extensions::new(&["php"]),
            Language::Lua => Extensions::new(&["lua"]),
            Language::Go => Extensions::new(&["go"]),
            Language::Java => Extensions::new(&["java"]),
            Language::Kotlin => Extensions::new(&["kt", "kts"]),
            Language::Scala => Extensions::new(&["scala"]),
            Language::Swift => Extensions::new(&["swift"]),
            Language::ObjectiveC => Extensions::new(&["m", "mm"]),
            Language::C => Extensions::new(&["c", "h"]),
            Language::Cpp => Extensions::new(&["cc", "cpp", "cxx", "hh", "hpp", "hxx"]),
            Language::CSharp => Extensions::new(&["cs"]),
            Language::Dart => Extensions::new(&["dart"]),
            Language::Haskell => Extensions::new(&["hs"]),
            Language::Elixir => Extensions::new(&["ex", "exs"]),
            Language::Erlang => Extensions::new(&["erl", "hrl"]),
            Language::Clojure => Extensions::new(&["clj", "cljs", "cljc"]),
            Language::OCaml => Extensions::new(&["ml", "mli"]),
            Language::FSharp => Extensions::new(&["fs", "fsx"]),
            Language::R => Extensions::new(&["r"]),
            Language::Julia => Extensions::new(&["jl"]),
            Language::Groovy => Extensions::new(&["groovy"]),
            Language::Zig => Extensions::new(&["zig"]),
            Language::JavaScript => Extensions::new(&["js", "mjs", "cjs", "jsx"]),
            Language::TypeScript => Extensions::new(&["ts", "tsx", "mts", "cts"]),
            Language::Html => Extensions::new(&["html", "htm"]),
            Language::Css => Extensions::new(&["css", "scss", "sass", "less"]),
            Language::Vue => Extensions::new(&["vue"]),
            Language::Svelte => Extensions::new(&["svelte"]),
            Language::Awk => Extensions::new(&["awk"]),
            Language::Sed => Extensions::new(&["sed"]),
            Language::Tcl => Extensions::new(&["tcl"]),
            Language::Nix => Extensions::new(&["nix"]),
            Language::Make => Extensions::new(&["mk"]),
            Language::Notebook => Extensions::new(&["ipynb"]),
            Language::Vim => Extensions::new(&["vim"]),
            Language::Lisp => Extensions::new(&["el", "lisp"]),
            Language::Scheme => Extensions::new(&["scm"]),
            Language::Racket => Extensions::new(&["rkt"]),
            Language::Nim => Extensions::new(&["nim"]),
            Language::VisualBasic => Extensions::new(&["vb", "vbs"]),
            Language::Solidity => Extensions::new(&["sol"]),
            Language::Assembly => Extensions::new(&["asm", "s"]),
            Language::D => Extensions::new(&["d"]),
            Language::Pascal => Extensions::new(&["pas"]),
            Language::Fortran => Extensions::new(&["f90"]),
            Language::Crystal => Extensions::new(&["cr"]),
            Language::V => Extensions::new(&["v"]),
            Language::Arduino => Extensions::new(&["ino"]),
            Language::Starlark => Extensions::new(&["bzl"]),
            Language::Astro => Extensions::new(&["astro"]),
            Language::Script => Extensions::new(&[]),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Python => formatter.write_str("Python"),
            Language::Shell => formatter.write_str("shell"),
            Language::Batch => formatter.write_str("batch"),
            Language::PowerShell => formatter.write_str("PowerShell"),
            Language::Perl => formatter.write_str("Perl"),
            Language::Ruby => formatter.write_str("Ruby"),
            Language::Php => formatter.write_str("PHP"),
            Language::Lua => formatter.write_str("Lua"),
            Language::Go => formatter.write_str("Go"),
            Language::Java => formatter.write_str("Java"),
            Language::Kotlin => formatter.write_str("Kotlin"),
            Language::Scala => formatter.write_str("Scala"),
            Language::Swift => formatter.write_str("Swift"),
            Language::ObjectiveC => formatter.write_str("Objective-C"),
            Language::C => formatter.write_str("C"),
            Language::Cpp => formatter.write_str("C++"),
            Language::CSharp => formatter.write_str("C#"),
            Language::Dart => formatter.write_str("Dart"),
            Language::Haskell => formatter.write_str("Haskell"),
            Language::Elixir => formatter.write_str("Elixir"),
            Language::Erlang => formatter.write_str("Erlang"),
            Language::Clojure => formatter.write_str("Clojure"),
            Language::OCaml => formatter.write_str("OCaml"),
            Language::FSharp => formatter.write_str("F#"),
            Language::R => formatter.write_str("R"),
            Language::Julia => formatter.write_str("Julia"),
            Language::Groovy => formatter.write_str("Groovy"),
            Language::Zig => formatter.write_str("Zig"),
            Language::JavaScript => formatter.write_str("JavaScript"),
            Language::TypeScript => formatter.write_str("TypeScript"),
            Language::Html => formatter.write_str("HTML"),
            Language::Css => formatter.write_str("CSS"),
            Language::Vue => formatter.write_str("Vue"),
            Language::Svelte => formatter.write_str("Svelte"),
            Language::Awk => formatter.write_str("awk"),
            Language::Sed => formatter.write_str("sed"),
            Language::Tcl => formatter.write_str("Tcl"),
            Language::Nix => formatter.write_str("Nix"),
            Language::Make => formatter.write_str("make"),
            Language::Notebook => formatter.write_str("notebook"),
            Language::Vim => formatter.write_str("Vim script"),
            Language::Lisp => formatter.write_str("Lisp"),
            Language::Scheme => formatter.write_str("Scheme"),
            Language::Racket => formatter.write_str("Racket"),
            Language::Nim => formatter.write_str("Nim"),
            Language::VisualBasic => formatter.write_str("Visual Basic"),
            Language::Solidity => formatter.write_str("Solidity"),
            Language::Assembly => formatter.write_str("assembly"),
            Language::D => formatter.write_str("D"),
            Language::Pascal => formatter.write_str("Pascal"),
            Language::Fortran => formatter.write_str("Fortran"),
            Language::Crystal => formatter.write_str("Crystal"),
            Language::V => formatter.write_str("V"),
            Language::Arduino => formatter.write_str("Arduino"),
            Language::Starlark => formatter.write_str("Starlark"),
            Language::Astro => formatter.write_str("Astro"),
            Language::Script => formatter.write_str("interpreted script"),
        }
    }
}
