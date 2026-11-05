use reserved_names::Reserved;

#[test]
fn every_listed_method_is_reserved() {
    let free = Reserved::METHODS
        .iter()
        .flat_map(|(_, names)| names.iter().copied())
        .filter(|name| Reserved::owner(name).is_none());
    assert_eq!(free.collect::<Vec<&str>>(), Vec::<&str>::new());
}

#[test]
fn the_derives_own_name_is_reserved() {
    assert_eq!(Reserved::owner("name"), Some("name"));
}

#[test]
fn the_names_every_type_has_are_reserved() {
    let blanket = ["type_id", "from", "try_from", "borrow_mut", "borrow", "into", "try_into"];
    let free = blanket.into_iter().filter(|name| Reserved::owner(name).is_none());
    assert_eq!(free.collect::<Vec<&str>>(), Vec::<&str>::new());
}

#[test]
fn an_ordinary_name_is_free() {
    assert_eq!(Reserved::owner("glow"), None);
}

#[test]
fn the_message_names_every_owner_and_the_derives_name() {
    let message = Reserved::message();
    let named = Reserved::METHODS.iter().filter(|(owner, _)| message.contains(owner)).count();
    assert_eq!(named, Reserved::METHODS.len());
    assert!(message.contains("`name`"));
}
