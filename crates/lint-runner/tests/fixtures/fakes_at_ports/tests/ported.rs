use corpus::ports::Clock;
#[derive(Debug)]
struct FakeClock;
impl Clock for FakeClock {
    fn tick(&self) {}
}
impl Drop for FakeClock {
    fn drop(&mut self) {}
}
