use syn::visit::Visit;
use syn::{Ident, Type, TypePath};

use lint_layout::Name;

use crate::wrappers::Wrappers;

pub struct TextMention {
    named: Vec<Ident>,
}

impl TextMention {
    const TEXT: [&'static str; 2] = ["String", "str"];

    pub fn in_type(ty: &Type, wrappers: &Wrappers) -> Option<Ident> {
        let mut mention = TextMention { named: Vec::new() };
        mention.visit_type(ty);
        mention.named.into_iter().find(|ident| {
            Self::TEXT.iter().any(|text| ident == text) || wrappers.holds_text(&Name::from(ident)).is_some()
        })
    }
}

impl<'ast> Visit<'ast> for TextMention {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.named.extend(
            Some(path)
                .filter(|path| path.qself.is_none())
                .and_then(|path| path.path.segments.last())
                .map(|segment| segment.ident.clone()),
        );
        syn::visit::visit_type_path(self, path);
    }
}
