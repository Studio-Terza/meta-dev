mod fix;

use std::ffi::OsStr;
use std::path::Path;

use fix::Fix;

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
    ];

    #[must_use]
    pub fn detect(path: &Path) -> Option<Language> {
        let extension = path.extension().map(OsStr::to_ascii_lowercase)?;
        let extension = extension.to_str()?;
        Self::ALL.iter().copied().find(|language| language.extensions().contains(&extension))
    }

    #[must_use]
    pub(crate) fn name(self) -> &'static str {
        match self {
            Language::Python => "Python",
            Language::Shell => "shell",
            Language::Batch => "batch",
            Language::PowerShell => "PowerShell",
            Language::Perl => "Perl",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Lua => "Lua",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::Swift => "Swift",
            Language::ObjectiveC => "Objective-C",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Dart => "Dart",
            Language::Haskell => "Haskell",
            Language::Elixir => "Elixir",
            Language::Erlang => "Erlang",
            Language::Clojure => "Clojure",
            Language::OCaml => "OCaml",
            Language::FSharp => "F#",
            Language::R => "R",
            Language::Julia => "Julia",
            Language::Groovy => "Groovy",
            Language::Zig => "Zig",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Vue => "Vue",
            Language::Svelte => "Svelte",
        }
    }

    #[must_use]
    pub fn fix(self) -> Fix {
        match self {
            Language::JavaScript
            | Language::TypeScript
            | Language::Html
            | Language::Css
            | Language::Vue
            | Language::Svelte => Fix::MoveToAssets,
            _ => Fix::Rewrite,
        }
    }

    fn extensions(self) -> &'static [&'static str] {
        match self {
            Language::Python => &["py", "pyw", "pyi"],
            Language::Shell => &["sh", "bash", "zsh", "fish", "ksh", "csh"],
            Language::Batch => &["bat", "cmd"],
            Language::PowerShell => &["ps1", "psm1"],
            Language::Perl => &["pl", "pm"],
            Language::Ruby => &["rb"],
            Language::Php => &["php"],
            Language::Lua => &["lua"],
            Language::Go => &["go"],
            Language::Java => &["java"],
            Language::Kotlin => &["kt", "kts"],
            Language::Scala => &["scala"],
            Language::Swift => &["swift"],
            Language::ObjectiveC => &["m", "mm"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cc", "cpp", "cxx", "hh", "hpp", "hxx"],
            Language::CSharp => &["cs"],
            Language::Dart => &["dart"],
            Language::Haskell => &["hs"],
            Language::Elixir => &["ex", "exs"],
            Language::Erlang => &["erl"],
            Language::Clojure => &["clj", "cljs"],
            Language::OCaml => &["ml", "mli"],
            Language::FSharp => &["fs", "fsx"],
            Language::R => &["r"],
            Language::Julia => &["jl"],
            Language::Groovy => &["groovy"],
            Language::Zig => &["zig"],
            Language::JavaScript => &["js", "mjs", "cjs", "jsx"],
            Language::TypeScript => &["ts", "tsx", "mts", "cts"],
            Language::Html => &["html", "htm"],
            Language::Css => &["css", "scss", "sass", "less"],
            Language::Vue => &["vue"],
            Language::Svelte => &["svelte"],
        }
    }
}
