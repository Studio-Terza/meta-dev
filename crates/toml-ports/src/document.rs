use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::codec::TomlFailure;

#[derive(Debug)]
pub struct Document<'text> {
    text: &'text str,
    table: DeTable<'text>,
}

#[derive(Clone, Copy, Debug)]
pub struct Table<'value, 'text> {
    text: &'text str,
    table: &'value DeTable<'text>,
}

#[derive(Clone, Copy, Debug)]
pub struct Value<'value, 'text> {
    text: &'text str,
    value: &'value DeValue<'text>,
}

#[derive(Clone, Copy, Debug)]
pub struct Key<'value> {
    name: &'value str,
    line: usize,
}

impl<'text> Document<'text> {
    pub fn read(text: &'text str) -> Result<Document<'text>, TomlFailure> {
        Ok(Document { text, table: DeTable::parse(text)?.into_inner() })
    }

    #[must_use]
    pub fn table(&self) -> Table<'_, 'text> {
        Table { text: self.text, table: &self.table }
    }
}

impl<'value, 'text> Table<'value, 'text> {
    #[must_use]
    pub fn get(self, key: &str) -> Option<Value<'value, 'text>> {
        self.table.get(key).map(|found| Value { text: self.text, value: found.get_ref() })
    }

    #[must_use]
    pub fn keys(self) -> Vec<Key<'value>> {
        self.table.keys().map(|key| Key { name: key.get_ref(), line: self.line(key) }).collect()
    }

    #[must_use]
    pub fn values(self) -> Vec<Value<'value, 'text>> {
        self.table.values().map(|value| Value { text: self.text, value: value.get_ref() }).collect()
    }

    fn line<Spanning>(self, spanned: &Spanned<Spanning>) -> usize {
        self.text.get(..spanned.span().start).map_or(0, |before| before.matches('\n').count()) + 1
    }
}

impl<'value, 'text> Value<'value, 'text> {
    #[must_use]
    pub fn table(self) -> Option<Table<'value, 'text>> {
        self.value.as_table().map(|table| Table { text: self.text, table })
    }

    #[must_use]
    pub fn text(self) -> Option<&'value str> {
        self.value.as_str()
    }

    #[must_use]
    pub fn flag(self) -> Option<bool> {
        self.value.as_bool()
    }

    #[must_use]
    pub fn array(self) -> Option<Vec<Value<'value, 'text>>> {
        self.value
            .as_array()
            .map(|listed| listed.iter().map(|item| Value { text: self.text, value: item.get_ref() }).collect())
    }

    #[must_use]
    pub fn each(self) -> Vec<Value<'value, 'text>> {
        match self.value {
            DeValue::Array(listed) => {
                listed.iter().map(|item| Value { text: self.text, value: item.get_ref() }).collect()
            }
            DeValue::String(_)
            | DeValue::Integer(_)
            | DeValue::Float(_)
            | DeValue::Boolean(_)
            | DeValue::Datetime(_)
            | DeValue::Table(_) => vec![self],
        }
    }
}

impl<'value> Key<'value> {
    #[must_use]
    pub fn name(self) -> &'value str {
        self.name
    }

    #[must_use]
    pub fn line(self) -> usize {
        self.line
    }
}
