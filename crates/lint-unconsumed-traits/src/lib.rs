mod consumers;
mod declaration;
mod declared;
mod finding;

use lint_core::{Headline, Parsed, Site, SourceFile, Violation};
use lint_file::{ContextLint, Reads};

use consumers::{Consumers, Consuming};
use declared::Declared;
use finding::Finding;

#[derive(Debug)]
pub struct UnconsumedTraits;

impl ContextLint for UnconsumedTraits {
    const HEADLINE: Headline = Headline::new(
        "a local trait with methods has a consumer in shipped code: a bound (generic, where clause, supertrait), impl Trait, dyn Trait, or a call to one of its provided methods",
    );
    const READS: Reads = Reads::Shipped;

    type Part<'head> = Consumers;
    type Context<'head> = Consumers;

    type Gathering<'head> = Consuming;

    fn gathering(parsed: Parsed<'_, '_>) -> Consuming {
        Consuming::from(parsed.file())
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, consumers: &Consumers) -> Vec<Violation<'file>> {
        Declared::of_file(file)
            .into_iter()
            .filter_map(|declaration| declaration.unconsumed(consumers))
            .map(|declaration| Violation::new(*AsRef::<Site>::as_ref(&declaration), Finding::from(&declaration)))
            .collect()
    }
}
