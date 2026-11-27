use corpus::Shipped;

const GOLDEN: &str = "tests/goldens/table.txt";
static LIMIT: u8 = 4;
const DERIVED: &str = GOLDEN;

#[derive(Debug)]
struct Case {
    golden: &'static str,
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    Plain,
}

#[derive(Debug)]
struct Unrelated;

const CASES: [Case; 1] = [Case { golden: GOLDEN, kind: Kind::Plain }];

#[test]
fn reads_the_table() {
    let read: Vec<&str> = CASES.iter().map(|case| case.golden).collect();
    assert_eq!(read, vec![GOLDEN]);
}

#[test]
fn spells_expected_values() {
    assert_eq!(Shipped.shipped(), 1);
}

#[test]
fn compares_flags() {
    let flags = [true];
    assert!(flags.iter().all(|flag| *flag));
}

#[test]
fn matches_a_literal() {
    match Shipped.shipped() {
        2 => (),
        _ => (),
    }
}

impl Unrelated {
    fn helped(&self) -> Shipped {
        Shipped
    }
}

trait Helping {
    fn help(&self);
}

type Alias = Shipped;

proptest! {
    #[test]
    fn generated(value in 0u8..4) {
        prop_assert_eq!(value, 3);
    }

    #[test]
    fn clean(value in 0u8..4) {
        prop_assert_eq!(value, value);
    }
}
