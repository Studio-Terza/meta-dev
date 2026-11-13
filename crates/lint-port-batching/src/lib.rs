mod holders;
mod judge;
mod reader;
mod spelling;

use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_fakes_at_ports::PortTraits;
use lint_file::{PrimedLint, Reads};
use lint_paths::Imports;

use holders::Holders;
use judge::Judge;
use reader::Reader;

#[derive(Debug)]
pub struct PortBatching;

#[derive(Debug)]
pub struct ShippedPorts<'head> {
    ports: PortTraits,
    imports: Imports<'head>,
}

#[derive(Default)]
struct Gathered<'head> {
    ports: Vec<PortTraits>,
    imports: Vec<Imports<'head>>,
}

impl<'head> Gathered<'head> {
    fn including(mut self, part: ShippedPorts<'head>) -> Gathered<'head> {
        self.ports.push(part.ports);
        self.imports.push(part.imports);
        self
    }
}

impl<'head> FromIterator<ShippedPorts<'head>> for ShippedPorts<'head> {
    fn from_iter<Parts: IntoIterator<Item = ShippedPorts<'head>>>(parts: Parts) -> ShippedPorts<'head> {
        let gathered = parts.into_iter().fold(Gathered::default(), Gathered::including);
        ShippedPorts { ports: gathered.ports.into_iter().collect(), imports: gathered.imports.into_iter().collect() }
    }
}

impl PrimedLint for PortBatching {
    const HEADLINE: Headline = Headline::new("ports are called once per batch, not once per element");
    const READS: Reads = Reads::Shipped;

    type PrimerPart<'head> = ShippedPorts<'head>;
    type Primer<'head> = ShippedPorts<'head>;
    type Part<'head> = Holders;
    type Context<'head> = Holders;

    fn primed<'head>(parsed: Parsed<'_, 'head>) -> ShippedPorts<'head> {
        let shipped = parsed.file().shipped().map(|_| parsed);
        ShippedPorts {
            ports: shipped.map(Parsed::file).map_or_default(PortTraits::from),
            imports: shipped.into_iter().map(Imports::from).collect(),
        }
    }

    fn contributed<'head>(parsed: Parsed<'_, 'head>, primer: &ShippedPorts<'head>) -> Holders {
        let judge = Judge::new(&primer.imports, &primer.ports);
        parsed.file().shipped().map_or_default(|_| Holders::read(parsed, &judge))
    }

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        primer: &ShippedPorts<'head>,
        holders: &Holders,
    ) -> Vec<Violation<'file>> {
        Reader::violations(file, &Judge::new(&primer.imports, &primer.ports), holders)
    }
}
