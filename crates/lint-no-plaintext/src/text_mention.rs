use syn::visit::Visit;
use syn::{Ident, Type, TypePath};

use lint_layout::Name;

use crate::wrappers::Wrappers;

pub struct TextMention<'ty> {
    named: Vec<&'ty Ident>,
}

impl<'ty> TextMention<'ty> {
    const TEXT: [&'static str; 2] = ["String", "str"];

    pub fn in_type(ty: &'ty Type, wrappers: &Wrappers) -> Option<&'ty Ident> {
        let mut mention = TextMention { named: Vec::new() };
        mention.visit_type(ty);
        mention.named.into_iter().find(|ident| {
            Self::TEXT.iter().any(|text| *ident == text) || wrappers.holds_text(&Name::from(*ident)).is_some()
        })
    }
}

impl<'ty> Visit<'ty> for TextMention<'ty> {
    fn visit_type_path(&mut self, path: &'ty TypePath) {
        self.named.extend(
            Some(path)
                .filter(|path| path.qself.is_none())
                .and_then(|path| path.path.segments.last())
                .map(|segment| &segment.ident),
        );
        syn::visit::visit_type_path(self, path);
    }
}
