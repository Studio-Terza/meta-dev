use std::fmt;

use graph::{Argument, Function, Model, ParamIndex, Read, Reader, Resolved};
use lint_layout::Name;

use super::dispatched::Dispatched;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadKey<'model> {
    implemented: &'model Resolved,
    method: &'model Name,
    taken: &'model Resolved,
    reader: &'model Reader,
    parameter: ParamIndex,
    fields: &'model [Name],
    arguments: &'model [Argument],
}

#[derive(Debug, Clone, Copy)]
struct Spelled<'model> {
    model: &'model Model,
    resolved: &'model Resolved,
}

#[derive(Debug, Clone, Copy)]
pub struct SpelledKey<'model> {
    model: &'model Model,
    key: ReadKey<'model>,
}

impl<'model> ReadKey<'model> {
    pub fn of_read(
        function: &'model Function,
        implemented: &'model Resolved,
        read: &'model Read,
    ) -> Option<ReadKey<'model>> {
        let ty = read.parameter().entry(function.signature().inputs())?.typed()?.dereferenced();
        let taken: &Resolved = ty.named()?.as_ref();
        Self::concrete(taken).map(|concrete| ReadKey {
            implemented,
            method: function.as_ref(),
            taken: concrete,
            reader: read.reader(),
            parameter: read.parameter(),
            fields: read.fields(),
            arguments: read.arguments(),
        })
    }

    pub fn run_together(&self, dispatched: &Dispatched<'_>) -> Option<()> {
        dispatched.covers(self.implemented).filter(|()| dispatched.constructs(self.reader).is_none())
    }

    pub fn spelled(self, model: &'model Model) -> SpelledKey<'model> {
        SpelledKey { model, key: self }
    }

    fn concrete(resolved: &Resolved) -> Option<&Resolved> {
        match resolved {
            Resolved::Param(_) | Resolved::Primitive(_) => None,
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Unknown(..) => Some(resolved),
        }
    }
}

impl fmt::Display for Spelled<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let declared = self.resolved.crate_item().and_then(|item| self.model.declared(item));
        let named = declared.map(|item| AsRef::<Name>::as_ref(item).to_string());
        let spelled = named.or_else(|| self.resolved.std().map(ToString::to_string));
        formatter.write_str(&spelled.unwrap_or_else(|| format!("{:?}", self.resolved)))
    }
}

impl fmt::Display for SpelledKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = self.key;
        let implemented = Spelled { model: self.model, resolved: key.implemented };
        let reader = Spelled { model: self.model, resolved: key.reader.resolved() };
        let member = key.reader.member().map_or_default(|name| format!("::{name}"));
        let path: Vec<String> = key.fields.iter().map(ToString::to_string).collect();
        let fields = key.fields.first().map_or_default(|_| format!(".{}", path.join(".")));
        write!(
            formatter,
            "{implemented}::{} is implemented by types that each read {reader}{member} on parameter {}{fields}",
            key.method, key.parameter
        )
    }
}
