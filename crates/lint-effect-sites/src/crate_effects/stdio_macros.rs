use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use lint_core::{Gate, Site, SourceFile, Walk};
use lint_layout::{LineNumber, Name, Whole};
use lint_paths::Ground;

use super::called::Called;
use super::effect::Effect;
use super::effect_site::EffectSite;
use super::vocabulary::Vocabulary;

pub(super) struct StdioMacros<'file> {
    file: &'file SourceFile,
    ground: Ground,
    found: Vec<EffectSite>,
}

impl<'file> StdioMacros<'file> {
    pub(super) fn found(file: &'file SourceFile) -> Vec<EffectSite> {
        let ground = if Gate::PORTS.admits(file) { Ground::Ports } else { Ground::Open };
        let mut walker = StdioMacros { file, ground, found: Vec::new() };
        walker.visit_file(file.syntax());
        walker.found
    }
}

impl<'ast> Visit<'ast> for StdioMacros<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.ground;
        self.ground = if Gate::PORTS.names(module) { Ground::Ports } else { outer };
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.ground = outer;
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || visit::visit_item_fn(self, function));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let printed = invocation
            .path
            .segments
            .last()
            .map(|last| (Name::from(&last.ident), LineNumber::from(last.ident.span())))
            .filter(|(name, _)| name.matches::<Whole>(&Vocabulary::MACROS))
            .map(|(name, line)| {
                EffectSite::new(Site::new(self.file.path(), line), Effect::Stdio, Called::Macro(name), self.ground)
            });
        self.found.extend(printed);
        visit::visit_macro(self, invocation);
    }
}
