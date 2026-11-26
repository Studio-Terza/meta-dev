mod content;
mod extensions;
mod fix;

use std::ffi::OsStr;
use std::path::Path;

use content::Content;
use extensions::Extensions;
pub(crate) use fix::Fix;
use lint_core::RepoFile;
use strum::{Display, VariantArray};

#[derive(Display, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Python,
    #[strum(serialize = "shell")]
    Shell,
    #[strum(serialize = "batch")]
    Batch,
    PowerShell,
    Perl,
    Ruby,
    #[strum(serialize = "PHP")]
    Php,
    Lua,
    Go,
    Java,
    Kotlin,
    Scala,
    Swift,
    #[strum(serialize = "Objective-C")]
    ObjectiveC,
    C,
    #[strum(serialize = "C++")]
    Cpp,
    #[strum(serialize = "C#")]
    CSharp,
    Dart,
    Haskell,
    Elixir,
    Erlang,
    Clojure,
    OCaml,
    #[strum(serialize = "F#")]
    FSharp,
    R,
    Julia,
    Groovy,
    Zig,
    JavaScript,
    TypeScript,
    #[strum(serialize = "HTML")]
    Html,
    #[strum(serialize = "CSS")]
    Css,
    Vue,
    Svelte,
    #[strum(serialize = "awk")]
    Awk,
    #[strum(serialize = "sed")]
    Sed,
    Tcl,
    Nix,
    #[strum(serialize = "make")]
    Make,
    #[strum(serialize = "notebook")]
    Notebook,
    #[strum(serialize = "Vim script")]
    Vim,
    Lisp,
    Scheme,
    Racket,
    Nim,
    #[strum(serialize = "Visual Basic")]
    VisualBasic,
    Solidity,
    #[strum(serialize = "assembly")]
    Assembly,
    D,
    Pascal,
    Fortran,
    Crystal,
    V,
    Arduino,
    Starlark,
    Astro,
    #[strum(serialize = "interpreted script")]
    Script,
}

impl Language {
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
        Self::VARIANTS.iter().copied().find(|language| language.extensions().holding(extension).is_some())
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
