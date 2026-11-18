mod consumers;
mod declared;

use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_sweep::Contributing;

use consumers::{Consumers, Consuming};
use declared::{Declarations, Declared};

#[derive(Debug)]
pub struct UnconsumedTraits;

impl ContextLint for UnconsumedTraits {
    const HEADLINE: Headline = Headline::new(
        "a local trait with methods has a consumer in shipped code: a bound (generic, where clause, supertrait), impl Trait, dyn Trait, or a call to one of its provided methods",
    );
    const READS: Reads = Reads::Shipped;

    type Part<'head> = Consumers;
    type Context<'head> = Consumers;

    type Gathering<'head> = Contributing<Consuming>;

    fn gathering(parsed: Parsed<'_, '_>) -> Contributing<Consuming> {
        Contributing::new(Consuming::from(<&SourceFile>::from(parsed)))
    }
}

impl WatchingLint for UnconsumedTraits {
    type Found<'file> = Declarations<'file>;
    type Watched<'file> = Contributing<Declared<'file>>;

    fn watched(file: &SourceFile) -> Contributing<Declared<'_>> {
        Contributing::new(Declared::from(file))
    }

    fn judged<'file, 'head: 'file>(
        _: &'file SourceFile,
        found: Declarations<'file>,
        consumers: &Consumers,
    ) -> Vec<Violation<'file>> {
        found.unconsumed(consumers)
    }
}
