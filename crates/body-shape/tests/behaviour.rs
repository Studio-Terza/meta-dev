use body_shape::Shape;

struct Shapes;

impl Shapes {
    fn of(text: &str) -> Shape {
        Shape::from(text.parse::<proc_macro2::TokenStream>().unwrap_or_default())
    }

    fn alike(pair: &[&str]) -> bool {
        match pair {
            [left, right] => Self::of(left).key() == Self::of(right).key(),
            _ => false,
        }
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
