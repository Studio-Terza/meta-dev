mod consumers;
mod declared;

use lint_core::{Headline, Parsed, SourceFile};
use lint_file::{ContextLint, Reads};
use lint_sweep::Judgement;

use consumers::{Consumers, Consuming};
use declared::Declared;

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

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        consumers: &'run Consumers,
    ) -> Judgement<'file, 'run> {
        Judgement::watching(Box::new(Declared::new(file, consumers)))
    }
}
