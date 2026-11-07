use syn::ItemMod;

use lint_core::{Gate, SourceFile};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ground {
    Core,
    Ports,
    Boundary,
    Store,
}

impl Ground {
    const GATED: [Self; 3] = [Self::Ports, Self::Boundary, Self::Store];

    pub(crate) fn gated(file: &SourceFile) -> Self {
        Self::GATED
            .into_iter()
            .find(|ground| ground.gate().and_then(|gate| gate.admits(file)).is_some())
            .unwrap_or(Self::Core)
    }

    pub(crate) fn within(self, module: &ItemMod) -> Self {
        Self::GATED
            .into_iter()
            .find(|ground| ground.gate().and_then(|gate| gate.names(module)).is_some())
            .unwrap_or(self)
    }

    const fn gate(self) -> Option<&'static Gate> {
        match self {
            Self::Core => None,
            Self::Ports => Some(&Gate::PORTS),
            Self::Boundary => Some(&Gate::BOUNDARY),
            Self::Store => Some(&Gate::STORE),
        }
    }
}
