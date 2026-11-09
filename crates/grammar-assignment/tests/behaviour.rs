#![deny(clippy::wildcard_enum_match_arm)]

use grammar_assignment::Emptied;
use laws::proptest::prelude::any;

laws::roundtrip!(
    an_emptied_assignment_reads_back_its_name,
    any::<Emptied>(),
    |emptied: &Emptied| format!("{}=", emptied.clone().into_inner()),
    |text: &String| text.parse::<Emptied>().ok(),
    Some
);

laws::table!(
    an_emptied_name_is_exactly_one_identifier,
    |name: &&str| Emptied::try_new((*name).to_owned()).ok().map(Emptied::into_inner),
    [
        "RUSTFLAGS" => Some("RUSTFLAGS".to_owned()),
        "_cache9" => Some("_cache9".to_owned()),
        "" => None,
        "=" => None,
        "A=" => None,
        "9A" => None,
        "a b" => None,
    ]
);
