use library::{Choice, Named, Shared, Used};

pub struct Reader;
impl Reader {
    fn read(&self, used: &Used) -> u8 {
        used.method();
        library::called();
        let _ = Choice::One;
        library::LIMIT + Used::SIZE
    }
}
#[derive(derive::Shown)]
struct Derived;
