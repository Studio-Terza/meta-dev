use body_shape::{Bounds, Erasure, Literals, Reading, Shape};
use equivalence::{Equivalence, Erased};
use syntax::Item;

struct Shapes;

impl Shapes {
    const BODIES: Equivalence = Equivalence::erasing(&[Erased::Locals, Erased::Literals]);
    const NAMES: Equivalence = Equivalence::erasing(&[Erased::Names, Erased::Literals]);
    const SPELLED: Equivalence = Equivalence::erasing(&[Erased::Locals]);
    const PREFIX: usize = 6;

    fn of(text: &str) -> Option<Shape> {
        Self::read(text, Self::BODIES).map(|reading| reading.shape())
    }

    fn read(text: &str, equivalence: Equivalence) -> Option<Reading> {
        let mut lexed = syntax_lexer::Lexed::parsed(&format!("fn f() {text}")).ok()?;
        let table = lexed.tokens_taken();
        let bounds = lexed.into_file().items.iter().find_map(Self::body)?;
        Some(Erasure::reading(&table, equivalence).read(bounds))
    }

    fn body(item: &Item) -> Option<Bounds> {
        match item {
            Item::Fn(function) => Some(Bounds::from(function.block.as_ref())),
            _ => None,
        }
    }

    fn alike_with_names_erased(pair: &[&str]) -> bool {
        pair.first().zip(pair.last()).filter(|_| pair.len() == 2).is_some_and(|(left, right)| {
            Self::read(left, Self::NAMES)
                .zip(Self::read(right, Self::NAMES))
                .is_some_and(|(ours, theirs)| ours.shape() == theirs.shape())
        })
    }

    fn alike_with_literals_kept(pair: &[&str]) -> bool {
        pair.first().zip(pair.last()).filter(|_| pair.len() == 2).is_some_and(|(left, right)| {
            Self::read(left, Self::SPELLED)
                .zip(Self::read(right, Self::SPELLED))
                .is_some_and(|(ours, theirs)| ours.shape() == theirs.shape())
        })
    }

    fn literals(text: &&str) -> String {
        let literals = Self::read(text, Self::BODIES).map_or_else(Literals::default, Reading::into_literals);
        literals.as_ref().iter().map(ToString::to_string).collect::<Vec<String>>().join(" ")
    }

    fn alike(pair: &[&str]) -> bool {
        pair.first().zip(pair.last()).filter(|_| pair.len() == 2).is_some_and(|(left, right)| {
            Self::of(left).zip(Self::of(right)).is_some_and(|(ours, theirs)| ours == theirs)
        })
    }

    fn size(text: &&str) -> String {
        Self::of(text).map_or_else(String::new, |shape| shape.size().to_string())
    }

    fn prefix(text: &str) -> Option<Reading> {
        let table = syntax_lexer::Lexed::parsed(text).ok()?.tokens_taken();
        let lexemes = table.lexemes();
        let bounds = Bounds::new(lexemes.first()?.place(), lexemes.get(Self::PREFIX)?.place());
        Some(Erasure::reading(&table, Self::BODIES).read(bounds))
    }

    fn prefix_size(text: &&str) -> String {
        Self::prefix(text).map_or_else(String::new, |reading| reading.shape().size().to_string())
    }

    fn prefixes_alike(pair: &[&str]) -> bool {
        let [left, right] = [pair.first(), pair.last()].map(|text| text.and_then(|found| Self::prefix(found)));
        left.zip(right).filter(|_| pair.len() == 2).is_some_and(|(first, second)| first.shape() == second.shape())
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
        ["{ f(-1) }", "{ f(1) }"] => false,
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
        "{ f(true, 2) }" => "true 2",
        "{ g(false) }" => "false",
    ]
);

laws::table!(
    a_range_reads_only_its_own_tokens_even_when_the_next_one_touches_it,
    Shapes::prefix_size,
    [
        "fn f() { a(b) }" => "6",
        "fn f() { a }" => "6",
    ]
);

laws::table!(
    a_local_at_the_end_of_a_range_is_not_named_by_a_call_after_the_range,
    Shapes::prefixes_alike,
    [
        ["fn f() { a(b) }", "fn f() { c(d) }"] => true,
        ["fn f() { a(b) }", "fn f() { c }"] => true,
        ["fn f() { a(b) }", "fn g() { a(b) }"] => false,
    ]
);
