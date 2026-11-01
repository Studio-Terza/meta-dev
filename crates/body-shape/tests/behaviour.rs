use body_shape::Shape;

const BASE: &str = "{ let total = 0; for item in items { if item > 3 { total += item * 2; } } total }";

struct Shapes;

impl Shapes {
    fn of(text: &str) -> Shape {
        Shape::from(text.parse::<proc_macro2::TokenStream>().unwrap_or_default())
    }

    fn alike(left: &str, right: &str) -> bool {
        Self::of(left).key() == Self::of(right).key()
    }
}

#[test]
fn local_names_and_literals_do_not_change_a_shape() {
    let renamed = "{ let sum = 10; for row in rows { if row > 7 { sum += row * 5; } } sum }";
    assert!(Shapes::alike(BASE, renamed));
}

#[test]
fn a_literal_keeps_its_place_even_when_negative() {
    assert!(Shapes::alike("{ f(1) }", "{ f(-1) }"));
    assert!(Shapes::alike("{ f(true) }", "{ f(7) }"));
}

#[test]
fn the_pattern_of_reuse_between_locals_is_part_of_the_shape() {
    assert!(!Shapes::alike("{ let a = 1; let b = 2; a + b }", "{ let a = 1; let b = 2; a + a }"));
}

#[test]
fn callees_methods_and_paths_keep_their_names() {
    assert!(!Shapes::alike("{ f(x) }", "{ g(x) }"));
    assert!(!Shapes::alike("{ x.f() }", "{ x.g() }"));
    assert!(!Shapes::alike("{ a::f }", "{ a::g }"));
    assert!(!Shapes::alike("{ f!(x) }", "{ g!(x) }"));
}

#[test]
fn types_variants_and_keywords_keep_their_names() {
    assert!(!Shapes::alike("{ Some(x) }", "{ Ok(x) }"));
    assert!(!Shapes::alike("{ let x = A; }", "{ let x = B; }"));
    assert!(!Shapes::alike("{ if x { y } }", "{ while x { y } }"));
}

#[test]
fn a_shape_counts_its_tokens_with_delimiters_and_without_minus_before_a_literal() {
    assert_eq!(Shapes::of("{ f(x) }").size().to_string(), "6");
    assert_eq!(Shapes::of("{ f(-1) }").size().to_string(), "6");
    assert_eq!(Shapes::of("{ a - b }").size().to_string(), "5");
}

#[test]
fn locals_are_numbered_by_first_use_not_by_name() {
    assert!(Shapes::alike("{ a + b }", "{ q + r }"));
    assert!(!Shapes::alike("{ a + b }", "{ q + q }"));
}
