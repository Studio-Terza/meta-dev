use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Fix, Site, Violation};
use lint_placement::CrateName;
use newtype::{Arbitrary, Display, Newtype};

use crate::file_flow::FileFlow;
use crate::function_flow::FunctionFlow;
use crate::verdict::Verdict;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Count(usize);

#[derive(Clone, Copy, Debug)]
enum Scope<'name> {
    Workspace,
    Crate(&'name CrateName),
}

impl fmt::Display for Scope<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Workspace => formatter.write_str("workspace"),
            Scope::Crate(name) => write!(formatter, "crate {name}"),
        }
    }
}

#[derive(Debug)]
pub struct Tally<'name> {
    scope: Scope<'name>,
    functions: Count,
    pipelines: Count,
    routers: Count,
    neither: Count,
}

impl Tally<'_> {
    pub fn of_flows<'file>(flows: &[FileFlow<'file>]) -> Vec<Violation<'file>> {
        let crates =
            flows.iter().fold(BTreeMap::new(), |mut crates: BTreeMap<CrateName, Vec<&FunctionFlow<'file>>>, flow| {
                crates.entry(flow.crate_name()).or_default().extend(flow.functions());
                crates
            });
        let all: Vec<&FunctionFlow<'file>> = crates.values().flatten().copied().collect();
        Self::placed(Scope::Workspace, &all)
            .into_iter()
            .chain(crates.iter().filter_map(|(name, functions)| Self::placed(Scope::Crate(name), functions)))
            .collect()
    }

    fn placed<'file>(scope: Scope<'_>, functions: &[&FunctionFlow<'file>]) -> Option<Violation<'file>> {
        functions.iter().map(|function| *AsRef::<Site<'file>>::as_ref(*function)).min().map(|site| {
            Violation::new(
                site,
                Tally {
                    scope,
                    functions: Count::new(functions.len()),
                    pipelines: Self::counted(functions, Verdict::Pipeline),
                    routers: Self::counted(functions, Verdict::Router),
                    neither: Self::counted(functions, Verdict::Neither),
                },
            )
        })
    }

    fn counted(functions: &[&FunctionFlow<'_>], verdict: Verdict) -> Count {
        Count::new(functions.iter().filter(|function| function.verdict() == verdict).count())
    }
}

impl fmt::Display for Tally<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} functions, {} pipelines, {} pure routers, {} neither",
            self.scope, self.functions, self.pipelines, self.routers, self.neither
        )
    }
}

impl lint_core::Finding for Tally<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "turn each function that is neither into a pipeline of let bindings and combinators, or into a pure router",
        )
    }
}
