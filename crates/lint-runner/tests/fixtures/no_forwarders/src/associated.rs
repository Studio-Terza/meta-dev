struct Maker;
impl Maker {
    fn make() -> u8 { 1 }
}
struct Caller;
impl Caller {
    fn make() -> u8 {
        Maker::make()
    }
}
