mod owners;
mod port_crate;
mod reserved_names;
mod roles;

use lint_core::{Headline, Lint, Report, SourceFile, Violation, WorkspaceLint};
use lint_effect_sites::EffectSites;
use lint_file::FileLint;
use workspace_metadata::{Graph, Metadata, Role};

use owners::Owners;
use port_crate::PortCrate;
use reserved_names::{CountedName, ReservedNames, SqlxName};
use roles::Roles;

#[derive(Debug)]
pub struct OwnedDependencies;

#[derive(Debug)]
pub struct KeyDependencyRoles;

#[derive(Debug)]
pub struct EffectsInFillers;

#[derive(Debug)]
pub struct CountedCalls;

#[derive(Debug)]
pub struct HiddenSqlx;

impl WorkspaceLint for OwnedDependencies {
    fn report<'file>(files: &'file [SourceFile], workspaces: &'file [Metadata]) -> Report<'file> {
        Report::new(
            Headline::new(
                "only the owning unit, which is a port together with the workspace proc-macro crates it depends on, depends directly on a package its [package.metadata.port] owns or on a package in that package's closure, unless the unit declares the package shared; and every owns entry names a package its unit depends on",
            ),
            Self::flagged(&Owners::declared(files), &workspaces.iter().collect()),
        )
    }
}

impl OwnedDependencies {
    fn flagged<'file>(owners: &Owners<'file>, graph: &Graph<'file>) -> Vec<Violation<'file>> {
        owners.foreign(graph.bypasses()).into_iter().chain(owners.stale(graph.stale())).collect()
    }
}

impl WorkspaceLint for KeyDependencyRoles {
    fn report<'file>(files: &'file [SourceFile], workspaces: &'file [Metadata]) -> Report<'file> {
        let graph: Graph<'file> = workspaces.iter().collect();
        let breaches = graph.role_breaches();
        let roles = Roles::new(files, Owners::declared(files));
        Report::new(
            Headline::new(
                "a key dependency is used only in its role: a port that names dependents is depended on only by crates declaring that role, a lexer crate is a leaf lexing one format, a crate that expands a port's macros declares the role the port names for its expanders, a lexer crate, the port naming lexers as dependents and every crate only they depend on deny at their crate root each lint the crate the lexers came from denied, and a workspace proc-macro reaches an owned package only through its own unit or a crate declaring the owning port's dependents role",
            ),
            [
                roles.breached(breaches.unroled()),
                roles.breached(breaches.unleafed()),
                roles.breached(breaches.reformatted()),
                roles.breached(breaches.unported()),
                roles.unexpanded(&graph.expansions()),
                roles.undenied(Role::Lexer, &graph.serving(Role::Lexer)),
            ]
            .concat(),
        )
    }
}

impl Lint for EffectsInFillers {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "in a crate that declares a #[port] trait, effects on files, environment, clock, network, processes, randomness and stdio happen only inside impl <Trait>Filler and impl Release blocks",
            ),
            EffectSites::crates(files)
                .values()
                .filter_map(|found| PortCrate::declared(found))
                .flat_map(|ported| ported.stray_effects())
                .collect(),
        )
    }
}

impl Lint for CountedCalls {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "counted_call and counted_acquire are named only in port-core and port-derive, so every op is counted through a generated filler",
            ),
            files.iter().flat_map(ReservedNames::<CountedName>::violations).collect(),
        )
    }
}

impl Lint for HiddenSqlx {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "__sqlx is named only in store-ports and store-ports-macros, so every query goes through store_ports::query! and no crate reaches sqlx through the re-export",
            ),
            files.iter().flat_map(ReservedNames::<SqlxName>::violations).collect(),
        )
    }
}
