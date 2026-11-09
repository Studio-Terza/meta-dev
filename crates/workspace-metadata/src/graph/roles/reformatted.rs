use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::crate_name::CrateName;
use crate::format::Format;
use crate::graph::DependencyKeys;
use crate::role::Role;

use super::super::Graph;
use super::breaching::Breaching;

#[derive(Debug)]
pub struct Reformatted<'meta> {
    breaching: Breaching<'meta>,
    format: &'meta Format,
    others: BTreeSet<&'meta CrateName>,
}

impl<'meta> Reformatted<'meta> {
    pub(super) fn found(graph: &Graph<'meta>) -> Vec<Reformatted<'meta>> {
        let formats: BTreeMap<&Format, BTreeSet<&CrateName>> = graph
            .declaring(Role::Lexer)
            .into_iter()
            .filter_map(|id| graph.package(id))
            .filter_map(|lexer| Some((lexer.format()?, lexer.name())))
            .fold(BTreeMap::new(), |mut formats, (format, name)| {
                formats.entry(format).or_insert_with(BTreeSet::new).insert(name);
                formats
            });
        formats
            .into_iter()
            .filter(|(_, names)| names.iter().nth(1).is_some())
            .flat_map(|(format, names)| Self::sharing(format, &names))
            .collect()
    }

    fn sharing(format: &'meta Format, names: &BTreeSet<&'meta CrateName>) -> Vec<Reformatted<'meta>> {
        names
            .iter()
            .map(|name| Reformatted {
                breaching: Breaching::new(name, DependencyKeys::default()),
                format,
                others: names.iter().copied().filter(|other| other != name).collect(),
            })
            .collect()
    }
}

impl<'meta> AsRef<Breaching<'meta>> for Reformatted<'meta> {
    fn as_ref(&self) -> &Breaching<'meta> {
        &self.breaching
    }
}

impl fmt::Display for Reformatted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let others: Vec<String> = self.others.iter().map(ToString::to_string).collect();
        write!(formatter, "lexes the format {}, which {} also lexes", self.format, others.join(" and "))
    }
}
