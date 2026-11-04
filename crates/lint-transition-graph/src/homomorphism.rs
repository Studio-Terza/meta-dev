use std::fmt;

use law_facts::Property;
use lint_core::{Fix, Headline, Report, Violation};
use lint_layout::Name;
use lint_placement::CrateName;

use crate::algebra::Algebras;
use crate::count::Count;
use crate::declarations::Declarations;
use crate::edge::Edge;
use crate::end::{End, Node};
use crate::form::Form;
use crate::graph_lint::GraphLint;
use crate::listed::Listed;
use crate::system::TypeSystem;

#[derive(Debug)]
pub struct Homomorphisms;

impl GraphLint for Homomorphisms {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let algebras = system.algebras();
        let declarations = system.declarations();
        let grouped = Edge::by_crate(system.edges().iter());
        Report::new(
            Headline::new(
                "report only, homomorphisms: a transition edge with one domain input and a different domain output, both with algebra operations, preserves the operations or is listed; declare it with laws::preserves! in a test of the same package, naming the input type, the output type and the edge function; listed per crate with the count of such edges and the undeclared ones with their sites",
            ),
            grouped
                .iter()
                .filter_map(|(name, edges)| CrateMorphisms::measured(name, edges, algebras, declarations).reported())
                .collect(),
        )
    }
}

#[derive(Debug)]
struct Morphism<'edge, 'found, 'file> {
    edge: &'edge Edge<'found, 'file>,
    source: Node<'found>,
    target: Node<'found>,
}

impl<'edge, 'found, 'file> Morphism<'edge, 'found, 'file> {
    fn of_edge(
        edge: &'edge Edge<'found, 'file>,
        algebras: &Algebras<'found, 'file>,
    ) -> Option<Morphism<'edge, 'found, 'file>> {
        let mut inputs = edge.ends().taken().copied();
        let source = inputs.next()?;
        let target = AsRef::<End<'found>>::as_ref(edge.ends()).node().copied()?;
        inputs
            .next()
            .map_or(Some(Morphism { edge, source, target }), |_| None)
            .filter(|_| matches!(edge.form(), Form::Transition))
            .filter(|morphism| morphism.source != morphism.target)
            .filter(|morphism| algebras.of_type(&morphism.source).is_some())
            .filter(|morphism| algebras.of_type(&morphism.target).is_some())
    }

    fn function(&self) -> &Name {
        AsRef::<Name>::as_ref(self.edge.transition().function())
    }

    fn declared(&self, declarations: &Declarations<'_>) -> Option<Property> {
        let names = [AsRef::<Name>::as_ref(&self.source), AsRef::<Name>::as_ref(&self.target), self.function()];
        declarations.properties_naming(self.edge.site().path(), &names).get(&Property::Preserves).copied()
    }
}

impl fmt::Display for Morphism<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}: {} at {}", self.source, self.target, self.function(), self.edge.site())
    }
}

#[derive(Debug)]
struct CrateMorphisms<'edge, 'found, 'file> {
    name: &'edge CrateName,
    in_scope: Count,
    declared: Count,
    undeclared: Vec<Morphism<'edge, 'found, 'file>>,
}

impl<'edge, 'found, 'file> CrateMorphisms<'edge, 'found, 'file> {
    fn measured(
        name: &'edge CrateName,
        edges: &[&'edge Edge<'found, 'file>],
        algebras: &Algebras<'found, 'file>,
        declarations: &Declarations<'_>,
    ) -> CrateMorphisms<'edge, 'found, 'file> {
        let morphisms: Vec<Morphism<'edge, 'found, 'file>> =
            edges.iter().filter_map(|edge| Morphism::of_edge(edge, algebras)).collect();
        let in_scope = Count::new(morphisms.len());
        let (declared, undeclared): (Vec<_>, Vec<_>) =
            morphisms.into_iter().partition(|morphism| morphism.declared(declarations).is_some());
        CrateMorphisms { name, in_scope, declared: Count::new(declared.len()), undeclared }
    }

    fn reported(self) -> Option<Violation<'file>> {
        self.undeclared.iter().map(|morphism| morphism.edge.site()).min().map(|site| Violation::new(site, &self))
    }
}

impl fmt::Display for CrateMorphisms<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let undeclared = Count::new(self.undeclared.len());
        write!(
            formatter,
            "crate {}: {} edges between algebraic types, {} declare preservation, {undeclared} do not; {}",
            self.name,
            self.in_scope,
            self.declared,
            Listed::<_, ';'>::joined(self.undeclared.iter()),
        )
    }
}

impl lint_core::Finding for CrateMorphisms<'_, '_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "declare the edge with laws::preserves! in a test of the same package, naming the input type, the output type and the edge function",
        )
    }
}
