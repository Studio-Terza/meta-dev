use corpus::ports::Clock;
trait Ticker {}
struct MockClock;
impl Clock for MockClock {
    fn tick(&self) {}
}
impl Ticker for MockClock {}
