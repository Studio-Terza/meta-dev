mod doubles;
mod kind;
mod port_traits;

use doubles::Doubles;
pub use kind::Kind;
use lint_core::{Headline, Parsed, SourceFile, TestFile, Violation};
use lint_file::{Concluded, PrimedFacts, PrimedLint, Reads};
use lint_paths::Imports;
pub use port_traits::PortTraits;

#[derive(Debug)]
pub struct FakesAtPorts;

#[derive(Debug)]
pub struct TestPorts<'head> {
    ports: PortTraits,
    imports: Imports<'head>,
}

#[derive(Default)]
struct Gathered<'head> {
    ports: Vec<PortTraits>,
    imports: Vec<Imports<'head>>,
}

impl<'head> Gathered<'head> {
    fn including(mut self, part: TestPorts<'head>) -> Gathered<'head> {
        self.ports.push(part.ports);
        self.imports.push(part.imports);
        self
    }
}

impl<'head> FromIterator<TestPorts<'head>> for TestPorts<'head> {
    fn from_iter<Parts: IntoIterator<Item = TestPorts<'head>>>(parts: Parts) -> TestPorts<'head> {
        let gathered = parts.into_iter().fold(Gathered::default(), Gathered::including);
        TestPorts { ports: gathered.ports.into_iter().collect(), imports: gathered.imports.into_iter().collect() }
    }
}

impl PrimedFacts for FakesAtPorts {
    type PrimerPart<'head> = TestPorts<'head>;
    type Primer<'head> = TestPorts<'head>;
    type Part<'head> = Doubles<'head>;
    type Context<'head> = Concluded<'head>;

    fn primed<'head>(parsed: Parsed<'_, 'head>) -> TestPorts<'head> {
        let file = parsed.file();
        let tested = TestFile::tested(file).map(|_| parsed);
        TestPorts {
            ports: file.shipped().map_or_default(PortTraits::from),
            imports: tested.into_iter().map(Imports::from).collect(),
        }
    }

    fn contributed<'head>(parsed: Parsed<'_, 'head>, primer: &TestPorts<'head>) -> Doubles<'head> {
        let tested = TestFile::tested(parsed.file()).map(|_| parsed);
        tested.map_or_default(|found| Doubles::scanned(found, &primer.imports, &primer.ports))
    }
}

impl PrimedLint for FakesAtPorts {
    const HEADLINE: Headline = Headline::new("test doubles stand in for ports");
    const READS: Reads = Reads::Every;

    fn judged<'file, 'head: 'file>(
        file: &'file SourceFile,
        _: &TestPorts<'head>,
        concluded: &Concluded<'head>,
    ) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
