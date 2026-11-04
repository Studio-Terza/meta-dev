use std::fmt;

use law_facts::{Law, LawPath, Property, Side};
use lint_core::{Fix, Headline, Report, Site, Violation};
use lint_layout::Name;
use lint_placement::CrateName;

use crate::algebra::Algebras;
use crate::count::Count;
use crate::declarations::{Checked, Declarations};
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
                "report only, homomorphisms: a transition edge with one domain input and a different domain output, both with algebra operations, preserves the operations or is listed; declare it with laws::preserves! in a test of the same package, calling an operation of each type and the edge function by path (Type::function) or in a closure; listed per crate with the count of such edges and the undeclared ones with their sites",
            ),
            grouped
                .iter()
                .filter_map(|(name, edges)| CrateMorphisms::measured(name, edges, algebras, declarations).reported())
                .chain(declarations.unresolved(algebras, Checked::Preservation))
                .collect(),
        )
    }
}

#[derive(Debug)]
struct Morphism<'edge, 'found, 'file> {
    edge: &'edge Edge<'found, 'file>,
    source: Node<'found>,
    target: Node<'found>,
    shortfalls: Vec<Shortfall<'edge, 'found, 'file>>,
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
            .map_or(Some(Morphism { edge, source, target, shortfalls: Vec::new() }), |_| None)
            .filter(|_| matches!(edge.form(), Form::Transition))
            .filter(|morphism| morphism.source != morphism.target)
            .filter(|morphism| algebras.of_type(&morphism.source).is_some())
            .filter(|morphism| algebras.of_type(&morphism.target).is_some())
    }

    fn function(&self) -> &Name {
        AsRef::<Name>::as_ref(self.edge.transition().function())
    }

    fn reached_by<'law>(&self, law: &'law Law<'_>) -> Option<&'law LawPath> {
        let function = self.function();
        law.routed_on(Side::Edge, AsRef::<Name>::as_ref(&self.source), function)
            .or_else(|| law.routed_on(Side::Edge, AsRef::<Name>::as_ref(&self.target), function))
    }

    fn undeclared(
        self,
        declarations: &'edge Declarations<'file>,
        algebras: &Algebras<'found, 'file>,
    ) -> Option<Morphism<'edge, 'found, 'file>> {
        let verdicts: Vec<Vec<Shortfall<'edge, 'found, 'file>>> = declarations
            .of_package(self.edge.site().path())
            .into_iter()
            .filter(|law| law.kind().property() == Some(Property::Preserves))
            .filter(|law| self.reached_by(law).is_some())
            .map(|law| Shortfall::of_law(law, self.source, self.target, algebras))
            .collect();
        let passing_law = verdicts.iter().position(Vec::is_empty);
        passing_law
            .map_or_else(|| Some(Morphism { shortfalls: verdicts.into_iter().flatten().collect(), ..self }), |_| None)
    }
}

impl fmt::Display for Morphism<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}: {} at {}", self.source, self.target, self.function(), self.edge.site())?;
        self.shortfalls.iter().try_for_each(|shortfall| write!(formatter, " ({shortfall})"))
    }
}

#[derive(Debug)]
struct Shortfall<'law, 'found, 'file> {
    law: &'law Law<'file>,
    side: Side,
    node: Node<'found>,
}

impl<'law, 'found, 'file> Shortfall<'law, 'found, 'file> {
    fn of_law(
        law: &'law Law<'file>,
        source: Node<'found>,
        target: Node<'found>,
        algebras: &Algebras<'found, 'file>,
    ) -> Vec<Shortfall<'law, 'found, 'file>> {
        [(Side::Input, source), (Side::Output, target)]
            .into_iter()
            .filter_map(|(side, node)| Shortfall::facing(law, side, node, algebras))
            .collect()
    }

    fn facing(
        law: &'law Law<'file>,
        side: Side,
        node: Node<'found>,
        algebras: &Algebras<'found, 'file>,
    ) -> Option<Shortfall<'law, 'found, 'file>> {
        algebras.operating(law, side, &node).map_or(Some(Shortfall { law, side, node }), |_| None)
    }
}

impl fmt::Display for Shortfall<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "law preserves! at {}: ", AsRef::<Site<'_>>::as_ref(self.law))?;
        self.described(formatter)
    }
}

impl Shortfall<'_, '_, '_> {
    fn described(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.law.on_side(self.side) {
            Some(route) => write!(formatter, "{route}, not an operation of {}", self.node),
            None => write!(formatter, "{} names no operation of {}", self.side, self.node),
        }
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
        declarations: &'edge Declarations<'file>,
    ) -> CrateMorphisms<'edge, 'found, 'file> {
        let morphisms: Vec<Morphism<'edge, 'found, 'file>> =
            edges.iter().filter_map(|edge| Morphism::of_edge(edge, algebras)).collect();
        let total = morphisms.len();
        let undeclared: Vec<Morphism<'edge, 'found, 'file>> =
            morphisms.into_iter().filter_map(|morphism| morphism.undeclared(declarations, algebras)).collect();
        let declared = Count::new(total - undeclared.len());
        CrateMorphisms { name, in_scope: Count::new(total), declared, undeclared }
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
            "declare the edge with laws::preserves! in a test of the same package, calling an operation of each type and the edge function by path (Type::function) or in a closure",
        )
    }
}
