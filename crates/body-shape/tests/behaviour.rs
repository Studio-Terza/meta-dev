use body_shape::{Reading, Shape};
use equivalence::{Equivalence, Erased, Laws};

struct Shapes;

impl Shapes {
    const BODIES: Equivalence = Equivalence::erasing(&[Erased::Locals, Erased::Literals], Laws::Ignored);
    const NAMES: Equivalence = Equivalence::erasing(&[Erased::Names, Erased::Literals], Laws::Ignored);
    const SPELLED: Equivalence = Equivalence::erasing(&[Erased::Locals], Laws::Ignored);

    fn of(text: &str) -> Shape {
        Self::read(text, Self::BODIES).shape()
    }

    fn read(text: &str, equivalence: Equivalence) -> Reading {
        Reading::erasing(text.parse::<proc_macro2::TokenStream>().unwrap_or_default(), equivalence)
    }

    fn alike_with_names_erased(pair: &[&str]) -> bool {
        pair.first().zip(pair.last()).filter(|_| pair.len() == 2).is_some_and(|(left, right)| {
            Self::read(left, Self::NAMES).shape() == Self::read(right, Self::NAMES).shape()
        })
    }

    fn alike_with_literals_kept(pair: &[&str]) -> bool {
        pair.first().zip(pair.last()).filter(|_| pair.len() == 2).is_some_and(|(left, right)| {
            Self::read(left, Self::SPELLED).shape() == Self::read(right, Self::SPELLED).shape()
        })
    }

    fn literals(text: &&str) -> String {
        let literals = Self::read(text, Self::BODIES).into_literals();
        literals.as_ref().iter().map(ToString::to_string).collect::<Vec<String>>().join(" ")
    }

    fn alike(pair: &[&str]) -> bool {
        pair.first()
            .zip(pair.last())
            .filter(|_| pair.len() == 2)
            .is_some_and(|(left, right)| Self::of(left) == Self::of(right))
    }

    fn size(text: &&str) -> String {
        Self::of(text).size().to_string()
    }
}

laws::table!(
    two_bodies_are_alike_when_only_names_and_literals_differ,
    Shapes::alike,
    [
        ["{ let total = 0; for item in items { if item > 3 { total += item * 2; } } total }", "{ let sum = 10; for row in rows { if row > 7 { sum += row * 5; } } sum }"] => true,
        ["{ f(1) }", "{ f(-1) }"] => true,
        ["{ f(true) }", "{ f(7) }"] => true,
        ["{ a + b }", "{ q + r }"] => true,
        ["{ a + b + a }", "{ p + q + p }"] => true,
        ["{ let a: u8 = 1; }", "{ let b: u8 = 1; }"] => true,
        ["{ S { field: x } }", "{ S { other: y } }"] => true,
    ]
);

laws::table!(
    two_bodies_differ_when_a_callee_a_path_a_type_a_keyword_or_the_reuse_of_a_local_differs,
    Shapes::alike,
    [
        ["{ let a = 1; let b = 2; a + b }", "{ let a = 1; let b = 2; a + a }"] => false,
        ["{ a + b }", "{ q + q }"] => false,
        ["{ a + b + c }", "{ a + b + b }"] => false,
        ["{ f(x) }", "{ g(x) }"] => false,
        ["{ x.f() }", "{ x.g() }"] => false,
        ["{ a::f }", "{ a::g }"] => false,
        ["{ a::f }", "{ b::f }"] => false,
        ["{ a::b::f }", "{ a::c::f }"] => false,
        ["{ f!(x) }", "{ g!(x) }"] => false,
        ["{ Some(x) }", "{ Ok(x) }"] => false,
        ["{ let x = A; }", "{ let x = B; }"] => false,
        ["{ if x { y } }", "{ while x { y } }"] => false,
    ]
);

laws::table!(
    a_shape_counts_its_tokens_with_delimiters_and_without_minus_before_a_literal,
    Shapes::size,
    [
        "{ f(x) }" => "6",
        "{ f(-1) }" => "6",
        "{ a - b }" => "5",
    ]
);

laws::table!(
    with_names_erased_callees_paths_and_types_are_holes_too,
    Shapes::alike_with_names_erased,
    [
        ["{ f(x) }", "{ g(x) }"] => true,
        ["{ x.f() }", "{ x.g() }"] => true,
        ["{ let x = A; }", "{ let x = B; }"] => true,
        ["{ f(x, y) }", "{ g(x, x) }"] => false,
        ["{ if x { y } }", "{ while x { y } }"] => false,
    ]
);

laws::table!(
    with_literals_kept_a_differing_literal_tells_bodies_apart,
    Shapes::alike_with_literals_kept,
    [
        ["{ f(1) }", "{ f(1) }"] => true,
        ["{ f(1) }", "{ f(2) }"] => false,
        ["{ f(true) }", "{ f(false) }"] => false,
        ["{ a + b }", "{ q + r }"] => true,
    ]
);

laws::table!(
    a_reading_spells_its_literals_in_order_with_their_sign,
    Shapes::literals,
    [
        "{ f(1, \"a\") }" => "1 \"a\"",
        "{ f(-1) }" => "-1",
        "{ f(x) }" => "",
        "{ a - 2 }" => "-2",
    ]
);
