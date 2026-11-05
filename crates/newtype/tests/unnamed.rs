use newtype::{Named, Unnamed, VariantArray};

#[derive(Named, VariantArray, Clone, Copy)]
enum Marker {
    #[name("marker")]
    Only,
}

laws::table!(unnamed_equality_follows_the_text, |pair: &(&str, &str)| Unnamed::<Marker>::new(pair.0) == Unnamed::<Marker>::new(pair.1), [
    ("a", "a") => true,
    ("", "") => true,
    ("a", "b") => false,
    ("a", "") => false,
]);

laws::table!(unnamed_debug_names_the_text, |text: &&str| format!("{:?}", Unnamed::<Marker>::new(text)), [
    "plum" => String::from("Unnamed { text: \"plum\" }"),
]);

laws::table!(unnamed_clone_keeps_the_text, |text: &&str| {
    let original = Unnamed::<Marker>::new(text);
    let copy = original.clone();
    (copy == original, format!("{copy:?}"))
}, [
    "plum" => (true, String::from("Unnamed { text: \"plum\" }")),
]);
