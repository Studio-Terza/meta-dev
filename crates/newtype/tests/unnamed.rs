use newtype::Unnamed;

struct Marker;

laws::table!(unnamed_equality_follows_the_text, |pair: &(&str, &str)| Unnamed::<Marker>::new(pair.0) == Unnamed::<Marker>::new(pair.1), [
    ("a", "a") => true,
    ("", "") => true,
    ("a", "b") => false,
    ("a", "") => false,
]);

laws::table!(unnamed_debug_names_the_text, |text: &&str| format!("{:?}", Unnamed::<Marker>::new(text)), [
    "plum" => String::from("Unnamed { text: \"plum\" }"),
]);
