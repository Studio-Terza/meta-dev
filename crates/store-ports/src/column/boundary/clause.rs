use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    column: String,
    predicate: Predicate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Predicate {
    Within(Vec<String>),
    AtMost(u32),
    Nullable(Box<Predicate>),
}

impl Clause {
    pub(super) fn within(column: &str, names: impl Iterator<Item = String>) -> Clause {
        Clause { column: column.to_owned(), predicate: Predicate::Within(names.collect()) }
    }

    pub(super) fn at_most(column: &str, most: u32) -> Clause {
        Clause { column: column.to_owned(), predicate: Predicate::AtMost(most) }
    }

    pub(super) fn nullable(self) -> Clause {
        Clause { predicate: Predicate::Nullable(Box::new(self.predicate)), column: self.column }
    }

    fn nulled(column: &str, inner: &Predicate, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{column} IS NULL OR (")?;
        inner.spelled(column, formatter)?;
        formatter.write_str(")")
    }
}

impl Predicate {
    fn listed(names: &[String], column: &str, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quoted: Vec<String> = names.iter().map(|name| format!("'{name}'")).collect();
        write!(formatter, "{column} IN ({})", quoted.join(", "))
    }

    fn spelled(&self, column: &str, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::Within(names) => Predicate::listed(names, column, formatter),
            Predicate::AtMost(most) => write!(formatter, "{column} BETWEEN 0 AND {most}"),
            Predicate::Nullable(inner) => Clause::nulled(column, inner, formatter),
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.predicate.spelled(&self.column, formatter)
    }
}
