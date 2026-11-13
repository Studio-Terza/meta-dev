mod coverage;
mod facts;
mod subject;

use coverage::Coverage;
use facts::{Facts, Profiles};
use law_facts::{Laws, Traits};
use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, PrimedLint, Reads};

#[derive(Debug)]
pub struct MandatoryLaws;

#[derive(Debug)]
pub struct LawFile<'head> {
    laws: Laws<'head>,
    profiles: Profiles<'head>,
}

#[derive(Default)]
struct Gathered<'head> {
    laws: Vec<Laws<'head>>,
    profiles: Vec<Profiles<'head>>,
}

impl<'head> Gathered<'head> {
    fn including(mut self, part: LawFile<'head>) -> Gathered<'head> {
        self.laws.push(part.laws);
        self.profiles.push(part.profiles);
        self
    }
}

impl<'head> FromIterator<LawFile<'head>> for Concluded<'head> {
    fn from_iter<Parts: IntoIterator<Item = LawFile<'head>>>(parts: Parts) -> Concluded<'head> {
        let gathered = parts.into_iter().fold(Gathered::default(), Gathered::including);
        let coverage = Coverage::from(gathered.laws.into_iter().collect::<Laws<'head>>());
        let facts: Facts<'head> = gathered.profiles.into_iter().collect();
        facts.unmet(&coverage).into_iter().collect()
    }
}

impl PrimedLint for MandatoryLaws {
    const HEADLINE: Headline = Headline::new("conversions have roundtrip laws");
    const READS: Reads = Reads::Every;

    type PrimerPart<'head> = Traits<'head>;
    type Primer<'head> = Traits<'head>;
    type Part<'head> = LawFile<'head>;
    type Context<'head> = Concluded<'head>;

    fn primed<'head>(parsed: Parsed<'_, 'head>) -> Traits<'head> {
        Traits::from(parsed)
    }

    fn contributed<'head>(parsed: Parsed<'_, 'head>, traits: &Traits<'head>) -> LawFile<'head> {
        LawFile { laws: Laws::read(parsed, traits), profiles: Profiles::from(parsed) }
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}
