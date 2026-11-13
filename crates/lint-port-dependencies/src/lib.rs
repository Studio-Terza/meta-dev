mod owners;
mod port_crate;
mod reserved_names;
mod roles;

use lint_core::{FileHead, Headline, Parsed, Report, SourceFile, Violation, WorkspaceLint};
use lint_file::{FileLint, PrimedFacts, PrimedLint, Reads};
use lint_paths::Imports;
use workspace_metadata::{Graph, Metadata, Role};

use owners::Owners;
use port_crate::PortedCrates;
use reserved_names::{CountedName, ReservedNames, SqlxName};
use roles::Roles;
pub use roles::{RoleFacts, RoleReading};

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
    type Facts<'head> = ();

    fn gathered(_: &[Parsed<'_, '_>]) {}

    fn report<'file, 'head: 'file>(
        heads: &'file [FileHead],
        workspaces: &'file [Metadata],
        (): &'file (),
    ) -> Report<'file> {
        Report::new(
            Headline::new(
                "only the owning unit, which is a port together with the workspace proc-macro crates it depends on, depends directly on a package its [package.metadata.port] owns or on a package in that package's closure, unless the unit declares the package shared; and every owns entry names a package its unit depends on",
            ),
            Self::flagged(&Owners::declared(heads), &workspaces.iter().collect()),
        )
    }
}

impl OwnedDependencies {
    fn flagged<'file>(owners: &Owners<'file>, graph: &Graph<'file>) -> Vec<Violation<'file>> {
        owners.foreign(graph.bypasses()).into_iter().chain(owners.stale(graph.stale())).collect()
    }
}

impl WorkspaceLint for KeyDependencyRoles {
    type Facts<'head> = RoleFacts<'head>;

    fn gathered<'head>(parsed: &[Parsed<'_, 'head>]) -> RoleFacts<'head> {
        let imports: Imports<'head> = parsed.iter().copied().map(RoleReading::primed).collect();
        parsed.iter().map(|found| RoleReading::contributed(*found, &imports)).collect()
    }

    fn report<'file, 'head: 'file>(
        heads: &'file [FileHead],
        workspaces: &'file [Metadata],
        facts: &'file RoleFacts<'head>,
    ) -> Report<'file> {
        let graph: Graph<'file> = workspaces.iter().collect();
        let breaches = graph.role_breaches();
        let roles = Roles::new(heads, facts);
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

impl PrimedFacts for EffectsInFillers {
    type PrimerPart<'head> = PortedCrates<'head>;
    type Primer<'head> = PortedCrates<'head>;
    type Part<'head> = ();
    type Context<'head> = ();

    fn primed<'head>(parsed: Parsed<'_, 'head>) -> PortedCrates<'head> {
        PortedCrates::from(parsed)
    }

    fn contributed<'head>(_: Parsed<'_, 'head>, _: &PortedCrates<'head>) {}
}

impl PrimedLint for EffectsInFillers {
    const HEADLINE: Headline = Headline::new(
        "in a crate that declares a #[port] trait, effects on files, environment, clock, network, processes, randomness and stdio happen only inside impl <Trait>Filler and impl Release blocks",
    );
    const READS: Reads = Reads::Shipped;

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        ported: &PortedCrates<'head>,
        (): &(),
    ) -> Vec<Violation<'file>> {
        ported.stray_effects(file)
    }
}

impl FileLint for CountedCalls {
    const HEADLINE: Headline = Headline::new(
        "counted_call and counted_acquire are named only in port-core and port-derive, so every op is counted through a generated filler",
    );
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Vec::from(ReservedNames::<CountedName>::from(file))
    }
}

impl FileLint for HiddenSqlx {
    const HEADLINE: Headline = Headline::new(
        "__sqlx is named only in store-ports and store-ports-macros, so every query goes through store_ports::query! and no crate reaches sqlx through the re-export",
    );
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Vec::from(ReservedNames::<SqlxName>::from(file))
    }
}
