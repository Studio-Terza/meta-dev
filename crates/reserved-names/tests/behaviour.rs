use reserved_names::{Key, Reserved};

#[test]
fn every_listed_name_is_refused_with_its_owner() {
    let listed = [
        ("name", "Named"),
        ("clone", "Clone"),
        ("clone_from", "Clone"),
        ("eq", "PartialEq"),
        ("ne", "PartialEq"),
        ("partial_cmp", "PartialOrd"),
        ("lt", "PartialOrd"),
        ("le", "PartialOrd"),
        ("gt", "PartialOrd"),
        ("ge", "PartialOrd"),
        ("cmp", "Ord"),
        ("max", "Ord"),
        ("min", "Ord"),
        ("clamp", "Ord"),
        ("hash", "Hash"),
        ("hash_slice", "Hash"),
        ("fmt", "Debug and Display"),
        ("to_string", "ToString"),
        ("from_str", "FromStr"),
        ("from", "From"),
        ("into", "Into"),
        ("try_from", "TryFrom"),
        ("try_into", "TryInto"),
        ("borrow", "Borrow"),
        ("borrow_mut", "BorrowMut"),
        ("as_ref", "AsRef"),
        ("to_owned", "ToOwned"),
        ("clone_into", "ToOwned"),
        ("type_id", "Any"),
        ("default", "Default"),
        ("VARIANTS", "VariantArray"),
    ];
    let wrong = listed.into_iter().filter(|(name, owner)| {
        let expected = format!(
            "`{name}` is a method of {owner}, and an accessor cannot take that name because it would shadow that method"
        );
        Reserved::refusal(Key::from(*name)).map(|found| found.to_string()) != Some(expected)
    });
    assert_eq!(wrong.collect::<Vec<(&str, &str)>>(), Vec::<(&str, &str)>::new());
}

#[test]
fn an_ordinary_name_is_free() {
    assert!(Reserved::refusal(Key::from("glow")).is_none());
}
