mod shape_index;

use lint_core::{Bound, Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads};

use shape_index::ShippedImpls;

#[derive(Debug)]
pub struct RepeatedMethods;

impl ContextLint for RepeatedMethods {
    const HEADLINE: Headline = Headline::new(
        "one public method shape on many types is repeated; use AsRef<T> for a stored field or From<&T> for a computed value, with a newtype, or a derive",
    );
    const READS: Reads = Reads::Shipped;

    type Part<'head> = ShippedImpls<'head>;
    type Context<'head> = Concluded<'head>;

    fn contributed<'head>(parsed: Parsed<'_, 'head>) -> ShippedImpls<'head> {
        ShippedImpls::from(parsed)
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}

impl RepeatedMethods {
    const TYPES_PER_SHAPE: Bound = Bound::Most(2);
}
