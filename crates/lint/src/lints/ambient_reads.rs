use proc_macro2::Ident;
use syn::{ItemUse, Path};

use crate::finding::{Findings, Found};
use crate::hooks::{Descent, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;
use crate::use_leaf::UseLeaf;

#[derive(Debug)]
pub(crate) struct AmbientReads {
    found: Vec<Found>,
    home: Option<()>,
}

impl AmbientReads {
    const HOMES: [&'static str; 2] = ["console_ports", "timer_ports"];
    const ENVIRONMENT: &'static str = "env";
    const READS: [&'static str; 9] =
        ["var", "var_os", "vars", "vars_os", "args", "args_os", "temp_dir", "home_dir", "current_dir"];
    const CLOCKS: [&'static str; 2] = ["SystemTime", "Instant"];
    const NOW: &'static str = "now";
    const FIX: &'static str = "read the environment through console-ports and the time through timer-ports' Clock";

    fn read(&self, owner: &Ident, read: &Ident) -> Option<Found> {
        let environment = owner == AmbientReads::ENVIRONMENT && AmbientReads::READS.iter().any(|name| read == name);
        let clock = read == AmbientReads::NOW && AmbientReads::CLOCKS.iter().any(|name| owner == name);
        let ambient = (environment || clock).then_some(()).filter(|()| self.home.is_none());
        ambient.map(|()| Found::spanned(read.span(), format_args!("reads {owner}::{read} directly"), AmbientReads::FIX))
    }
}

impl Hooks for AmbientReads {
    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let found: Vec<Found> = UseLeaf::of_tree(&item.tree)
            .iter()
            .filter_map(|leaf| self.read(leaf.path().last()?, leaf.end().original()?))
            .collect();
        self.found.extend(found);
        Descent::Over
    }

    fn path(&mut self, path: &Path) -> Descent {
        let segments = path.segments.iter().map(|segment| &segment.ident);
        let found: Vec<Found> =
            segments.clone().zip(segments.skip(1)).filter_map(|(owner, read)| self.read(owner, read)).collect();
        self.found.extend(found);
        Descent::Into
    }
}

impl<'file> Lint<'file> for AmbientReads {
    const HEADLINE: &'static str = "only console-ports reads the environment and only timer-ports reads the clock";
    const READS: Reads = Reads::Every;

    type Fact = Findings;

    fn new(source: &'file Source) -> AmbientReads {
        let crate_name = source.package().crate_name();
        let home = AmbientReads::HOMES.iter().any(|home| *home == crate_name).then_some(());
        AmbientReads { found: Vec::new(), home }
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(AmbientReads::HEADLINE, source.shown(), self.found)
    }
}
