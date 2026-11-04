mod attached;

use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Fix, Headline, Report, Site, Violation};
use lint_layout::Name;
use lint_placement::CrateName;

use crate::algebra::Operation;
use crate::count::Count;
use crate::end::Node;
use crate::graph_lint::GraphLint;
use crate::system::TypeSystem;
use attached::Attached;

#[derive(Debug)]
pub struct AlgebraLaws;

impl GraphLint for AlgebraLaws {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let algebras = system.algebras();
        let attached = Attached::declared(system.files(), algebras);
        let by_crate = algebras.operations().fold(
            BTreeMap::new(),
            |mut crates: BTreeMap<&CrateName, Vec<&Operation<'_, 'file>>>, operation| {
                crates.entry(AsRef::<CrateName>::as_ref(operation)).or_default().push(operation);
                crates
            },
        );
        Report::new(
            Headline::new(
                "report only, algebra laws: every algebra operation (value inputs and output one type, acted on by any external parameter) and partial operation (Option or Result of the type) declares a law, with laws::associative!, commutative!, identity!, inverse!, idempotent!, absorbing! or distributive! in a test of the same package that names the type and the function; operations a foreign trait fixes are left out unless it is a std::ops operator, and every shipped same-type operation counts whatever its visibility; listed per crate and type",
            ),
            by_crate
                .iter()
                .filter_map(|(name, operations)| CrateLaws::measured(name, operations, &attached).reported())
                .collect(),
        )
    }
}

#[derive(Debug)]
struct CrateLaws<'op, 'file> {
    name: &'op CrateName,
    first: Option<Site<'file>>,
    algebraic: Count,
    unlawed: BTreeMap<&'op Node<'op>, Vec<&'op Name>>,
}

impl<'op, 'file> CrateLaws<'op, 'file> {
    fn measured<'found: 'op>(
        name: &'op CrateName,
        operations: &[&'op Operation<'found, 'file>],
        attached: &Attached<'found>,
    ) -> CrateLaws<'op, 'file> {
        let unlawed: Vec<&&'op Operation<'found, 'file>> =
            operations.iter().filter(|operation| attached.of_operation(operation).is_empty()).collect();
        CrateLaws {
            name,
            first: unlawed.iter().map(|operation| *AsRef::<Site<'file>>::as_ref(operation)).min(),
            algebraic: Count::new(operations.len()),
            unlawed: unlawed.iter().fold(BTreeMap::new(), |mut types, operation| {
                types.entry(operation.acting().subject()).or_default().push(operation.function());
                types
            }),
        }
    }

    fn reported(self) -> Option<Violation<'file>> {
        self.first.map(|site| Violation::new(site, &self))
    }

    fn missing(&self) -> Count {
        Count::new(self.unlawed.values().map(Vec::len).sum())
    }
}

impl fmt::Display for CrateLaws<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "crate {}: {} of {} algebra operations declare no law",
            self.name,
            self.missing(),
            self.algebraic
        )?;
        self.unlawed.iter().try_for_each(|(node, functions)| {
            write!(formatter, "; {node}: {}", functions.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "))
        })
    }
}

impl lint_core::Finding for CrateLaws<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "declare the law with laws::associative!, commutative!, identity!, inverse!, idempotent!, absorbing! or distributive! in a test of the same package, naming the type and the function",
        )
    }
}
