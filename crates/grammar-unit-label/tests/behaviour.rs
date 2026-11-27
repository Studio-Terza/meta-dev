#![deny(clippy::wildcard_enum_match_arm)]

use grammar::{Ignoring, Unlexed};
use grammar_unit_label::Fixed;
use strum::VariantArray;

#[test]
fn a_fixed_label_is_displayed_and_parsed_back() {
    let read: Vec<Option<Fixed>> = Fixed::VARIANTS
        .iter()
        .map(|fixed| fixed.to_string().parse::<Fixed>().ignoring(Unlexed::NotThisGrammar))
        .collect();
    assert_eq!(read, Fixed::VARIANTS.iter().copied().map(Some).collect::<Vec<Option<Fixed>>>());
}
