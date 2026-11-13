mod shape_index;

use lint_core::{Bound, Headline, Parsed, SourceFile};
use lint_file::{Concluded, ContextLint, Reads};
use lint_sweep::Judgement;

use shape_index::{ImplGathering, ShippedImpls};

#[derive(Debug)]
pub struct RepeatedMethods;

impl ContextLint for RepeatedMethods {
    const HEADLINE: Headline = Headline::new(
        "one public method shape on many types is repeated; use AsRef<T> for a stored field or From<&T> for a computed value, with a newtype, or a derive",
    );
    const READS: Reads = Reads::Shipped;

    type Part<'head> = ShippedImpls<'head>;
    type Context<'head> = Concluded<'head>;

    type Gathering<'head> = ImplGathering<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> ImplGathering<'head> {
        ImplGathering::from(parsed)
    }

    fn judgement<'run, 'file: 'run, 'head: 'file>(
        file: &'file SourceFile,
        concluded: &'run Concluded<'head>,
    ) -> Judgement<'file, 'run> {
        Judgement::found(concluded.found_in(file))
    }
}

impl RepeatedMethods {
    const TYPES_PER_SHAPE: Bound = Bound::Most(2);
}
