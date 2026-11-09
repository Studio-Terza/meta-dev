use std::ffi::OsString;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Scope {
    Default,
    Workspace,
    Packages(Vec<String>),
}

impl Scope {
    const END: &'static str = "--";
    const WORKSPACE: [&'static str; 2] = ["--workspace", "--all"];
    const PACKAGE: [&'static str; 2] = ["-p", "--package"];

    fn named(words: &[&str]) -> Vec<String> {
        words
            .iter()
            .zip(words.iter().skip(1))
            .filter(|(flag, _)| Self::PACKAGE.contains(flag))
            .map(|(_, name)| (*name).to_owned())
            .collect()
    }
}

impl From<&[OsString]> for Scope {
    fn from(arguments: &[OsString]) -> Scope {
        let words: Vec<&str> =
            arguments.iter().map_while(|argument| argument.to_str()).take_while(|word| *word != Self::END).collect();
        let packages = Self::named(&words);
        match (words.iter().any(|word| Self::WORKSPACE.contains(word)), packages.is_empty()) {
            (true, _) => Scope::Workspace,
            (false, false) => Scope::Packages(packages),
            (false, true) => Scope::Default,
        }
    }
}
