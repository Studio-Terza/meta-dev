use syn::ItemMod;

use lint_core::{FileHead, Gate};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ground {
    Core,
    Ports,
    Boundary,
    Store,
}

impl Ground {
    const GATED: [Ground; 3] = [Ground::Ports, Ground::Boundary, Ground::Store];

    pub(crate) fn gated(file: &FileHead) -> Ground {
        Self::GATED
            .into_iter()
            .find(|ground| ground.gate().and_then(|gate| gate.admits(file)).is_some())
            .unwrap_or(Ground::Core)
    }

    pub(crate) fn within(self, module: &ItemMod) -> Ground {
        Self::GATED
            .into_iter()
            .find(|ground| ground.gate().and_then(|gate| gate.names(module)).is_some())
            .unwrap_or(self)
    }

    fn gate(self) -> Option<&'static Gate> {
        match self {
            Ground::Core => None,
            Ground::Ports => Some(&Gate::PORTS),
            Ground::Boundary => Some(&Gate::BOUNDARY),
            Ground::Store => Some(&Gate::STORE),
        }
    }
}
