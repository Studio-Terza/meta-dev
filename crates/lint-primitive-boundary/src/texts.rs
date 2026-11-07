use code_model::{Call, Callee, ItemPath, Model, Placed};
use lint_core::Violation;
use lint_layout::{Name, Whole, Word, words};

use crate::finding::Finding;
use crate::lexicon::Lone;
use crate::scope::Scope;

#[derive(Debug)]
pub struct Texts<'read, 'model> {
    scope: &'read Scope<'model>,
    string_from: ItemPath,
}

impl<'read, 'model> Texts<'read, 'model> {
    const TEXT_METHODS: [Word<'static>; 10] = words![
        "split_whitespace",
        "split_once",
        "trim",
        "trim_start",
        "trim_end",
        "chars",
        "char_indices",
        "split_at_checked",
        "to_string",
        "push_str",
    ];
    const STRING_FROM: [Word<'static>; 2] = words!["String", "from"];
    const FORMAT: [Word<'static>; 1] = words!["format"];

    pub fn new(scope: &'read Scope<'model>) -> Self {
        Texts { scope, string_from: Self::STRING_FROM.into_iter().collect() }
    }

    pub fn found(&self, model: &'model Model) -> Vec<Violation<'model>> {
        model.calls().filter_map(|placed| self.sighted(placed)).collect()
    }

    fn sighted(&self, placed: Placed<'model, &'model Call>) -> Option<Violation<'model>> {
        let handler = self.handler(placed.entity().callee());
        let call = handler.as_ref().and_then(|_| self.scope.call(placed.entity(), placed.site().path()));
        call.and(handler).map(|handler| Violation::new(placed.site(), Finding::TextOutsideDoor { handler }))
    }

    fn handler(&self, callee: &Callee) -> Option<Name> {
        let method = callee.method().and_then(|name| name.matches::<Whole>(&Self::TEXT_METHODS).map(|_| name));
        let converted = callee.path().filter(|written| **written == self.string_from).and_then(ItemPath::last);
        let invoked = callee.invoked().map(AsRef::<ItemPath>::as_ref);
        let formatted = invoked
            .and_then(|written| Lone::within(written).and(written.last()))
            .and_then(|last| last.matches::<Whole>(&Self::FORMAT).map(|_| last));
        method.or(converted).or(formatted).cloned()
    }
}
