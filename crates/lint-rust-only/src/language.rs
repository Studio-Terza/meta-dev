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
    const ALL: &'static [Self] = &[
        Self::Python,
        Self::Shell,
        Self::Batch,
        Self::PowerShell,
        Self::Perl,
        Self::Ruby,
        Self::Php,
        Self::Lua,
        Self::Go,
        Self::Java,
        Self::Kotlin,
        Self::Scala,
        Self::Swift,
        Self::ObjectiveC,
        Self::C,
        Self::Cpp,
        Self::CSharp,
        Self::Dart,
        Self::Haskell,
        Self::Elixir,
        Self::Erlang,
        Self::Clojure,
        Self::OCaml,
        Self::FSharp,
        Self::R,
        Self::Julia,
        Self::Groovy,
        Self::Zig,
        Self::JavaScript,
        Self::TypeScript,
        Self::Html,
        Self::Css,
        Self::Vue,
        Self::Svelte,
        Self::Awk,
        Self::Sed,
        Self::Tcl,
        Self::Nix,
        Self::Make,
        Self::Notebook,
        Self::Vim,
        Self::Lisp,
        Self::Scheme,
        Self::Racket,
        Self::Nim,
        Self::VisualBasic,
        Self::Solidity,
        Self::Assembly,
        Self::D,
        Self::Pascal,
        Self::Fortran,
        Self::Crystal,
        Self::V,
        Self::Arduino,
        Self::Starlark,
        Self::Astro,
        Self::Script,
    ];

    const RUST: &'static str = "rs";

    #[must_use]
    pub fn detect(file: &RepoFile) -> Option<Self> {
        Some(file.shown())
            .filter(|path| path.extension().is_none_or(|extension| extension.to_ascii_lowercase() != *Self::RUST))
            .and_then(|path| {
                path.extension()
                    .and_then(Self::by_extension)
                    .or_else(|| path.file_stem().map(Path::new).and_then(Path::extension).and_then(Self::by_extension))
                    .or_else(|| Content::language_of(file.contents()))
            })
    }

    fn by_extension(extension: &OsStr) -> Option<Self> {
        Self::ALL.iter().copied().find(|language| language.extensions().holding(extension).is_some())
    }

    #[must_use]
    pub const fn fix(self) -> Fix {
        match self {
            Self::JavaScript | Self::TypeScript | Self::Html | Self::Css | Self::Vue | Self::Svelte | Self::Astro => {
                Fix::MoveToAssets
            }
            _ => Fix::Rewrite,
        }
    }

    const fn extensions(self) -> Extensions {
        match self {
            Self::Python => Extensions::new(&["py", "pyw", "pyi", "pyx"]),
            Self::Shell => Extensions::new(&["sh", "bash", "zsh", "fish", "ksh", "csh", "dash"]),
            Self::Batch => Extensions::new(&["bat", "cmd"]),
            Self::PowerShell => Extensions::new(&["ps1", "psm1"]),
            Self::Perl => Extensions::new(&["pl", "pm", "t"]),
            Self::Ruby => Extensions::new(&["rb", "rake", "erb"]),
            Self::Php => Extensions::new(&["php"]),
            Self::Lua => Extensions::new(&["lua"]),
            Self::Go => Extensions::new(&["go"]),
            Self::Java => Extensions::new(&["java"]),
            Self::Kotlin => Extensions::new(&["kt", "kts"]),
            Self::Scala => Extensions::new(&["scala"]),
            Self::Swift => Extensions::new(&["swift"]),
            Self::ObjectiveC => Extensions::new(&["m", "mm"]),
            Self::C => Extensions::new(&["c", "h"]),
            Self::Cpp => Extensions::new(&["cc", "cpp", "cxx", "hh", "hpp", "hxx"]),
            Self::CSharp => Extensions::new(&["cs"]),
            Self::Dart => Extensions::new(&["dart"]),
            Self::Haskell => Extensions::new(&["hs"]),
            Self::Elixir => Extensions::new(&["ex", "exs"]),
            Self::Erlang => Extensions::new(&["erl", "hrl"]),
            Self::Clojure => Extensions::new(&["clj", "cljs", "cljc"]),
            Self::OCaml => Extensions::new(&["ml", "mli"]),
            Self::FSharp => Extensions::new(&["fs", "fsx"]),
            Self::R => Extensions::new(&["r"]),
            Self::Julia => Extensions::new(&["jl"]),
            Self::Groovy => Extensions::new(&["groovy"]),
            Self::Zig => Extensions::new(&["zig"]),
            Self::JavaScript => Extensions::new(&["js", "mjs", "cjs", "jsx"]),
            Self::TypeScript => Extensions::new(&["ts", "tsx", "mts", "cts"]),
            Self::Html => Extensions::new(&["html", "htm"]),
            Self::Css => Extensions::new(&["css", "scss", "sass", "less"]),
            Self::Vue => Extensions::new(&["vue"]),
            Self::Svelte => Extensions::new(&["svelte"]),
            Self::Awk => Extensions::new(&["awk"]),
            Self::Sed => Extensions::new(&["sed"]),
            Self::Tcl => Extensions::new(&["tcl"]),
            Self::Nix => Extensions::new(&["nix"]),
            Self::Make => Extensions::new(&["mk"]),
            Self::Notebook => Extensions::new(&["ipynb"]),
            Self::Vim => Extensions::new(&["vim"]),
            Self::Lisp => Extensions::new(&["el", "lisp"]),
            Self::Scheme => Extensions::new(&["scm"]),
            Self::Racket => Extensions::new(&["rkt"]),
            Self::Nim => Extensions::new(&["nim"]),
            Self::VisualBasic => Extensions::new(&["vb", "vbs"]),
            Self::Solidity => Extensions::new(&["sol"]),
            Self::Assembly => Extensions::new(&["asm", "s"]),
            Self::D => Extensions::new(&["d"]),
            Self::Pascal => Extensions::new(&["pas"]),
            Self::Fortran => Extensions::new(&["f90"]),
            Self::Crystal => Extensions::new(&["cr"]),
            Self::V => Extensions::new(&["v"]),
            Self::Arduino => Extensions::new(&["ino"]),
            Self::Starlark => Extensions::new(&["bzl"]),
            Self::Astro => Extensions::new(&["astro"]),
            Self::Script => Extensions::new(&[]),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => formatter.write_str("Python"),
            Self::Shell => formatter.write_str("shell"),
            Self::Batch => formatter.write_str("batch"),
            Self::PowerShell => formatter.write_str("PowerShell"),
            Self::Perl => formatter.write_str("Perl"),
            Self::Ruby => formatter.write_str("Ruby"),
            Self::Php => formatter.write_str("PHP"),
            Self::Lua => formatter.write_str("Lua"),
            Self::Go => formatter.write_str("Go"),
            Self::Java => formatter.write_str("Java"),
            Self::Kotlin => formatter.write_str("Kotlin"),
            Self::Scala => formatter.write_str("Scala"),
            Self::Swift => formatter.write_str("Swift"),
            Self::ObjectiveC => formatter.write_str("Objective-C"),
            Self::C => formatter.write_str("C"),
            Self::Cpp => formatter.write_str("C++"),
            Self::CSharp => formatter.write_str("C#"),
            Self::Dart => formatter.write_str("Dart"),
            Self::Haskell => formatter.write_str("Haskell"),
            Self::Elixir => formatter.write_str("Elixir"),
            Self::Erlang => formatter.write_str("Erlang"),
            Self::Clojure => formatter.write_str("Clojure"),
            Self::OCaml => formatter.write_str("OCaml"),
            Self::FSharp => formatter.write_str("F#"),
            Self::R => formatter.write_str("R"),
            Self::Julia => formatter.write_str("Julia"),
            Self::Groovy => formatter.write_str("Groovy"),
            Self::Zig => formatter.write_str("Zig"),
            Self::JavaScript => formatter.write_str("JavaScript"),
            Self::TypeScript => formatter.write_str("TypeScript"),
            Self::Html => formatter.write_str("HTML"),
            Self::Css => formatter.write_str("CSS"),
            Self::Vue => formatter.write_str("Vue"),
            Self::Svelte => formatter.write_str("Svelte"),
            Self::Awk => formatter.write_str("awk"),
            Self::Sed => formatter.write_str("sed"),
            Self::Tcl => formatter.write_str("Tcl"),
            Self::Nix => formatter.write_str("Nix"),
            Self::Make => formatter.write_str("make"),
            Self::Notebook => formatter.write_str("notebook"),
            Self::Vim => formatter.write_str("Vim script"),
            Self::Lisp => formatter.write_str("Lisp"),
            Self::Scheme => formatter.write_str("Scheme"),
            Self::Racket => formatter.write_str("Racket"),
            Self::Nim => formatter.write_str("Nim"),
            Self::VisualBasic => formatter.write_str("Visual Basic"),
            Self::Solidity => formatter.write_str("Solidity"),
            Self::Assembly => formatter.write_str("assembly"),
            Self::D => formatter.write_str("D"),
            Self::Pascal => formatter.write_str("Pascal"),
            Self::Fortran => formatter.write_str("Fortran"),
            Self::Crystal => formatter.write_str("Crystal"),
            Self::V => formatter.write_str("V"),
            Self::Arduino => formatter.write_str("Arduino"),
            Self::Starlark => formatter.write_str("Starlark"),
            Self::Astro => formatter.write_str("Astro"),
            Self::Script => formatter.write_str("interpreted script"),
        }
    }
}
