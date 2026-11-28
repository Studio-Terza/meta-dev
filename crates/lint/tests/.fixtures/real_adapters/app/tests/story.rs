use clock_ports::{ClockFiller, Fake, Files as Disk, SystemClock};
impl ClockFiller for S {} impl Disk for S {} impl clock_ports::Clock for S {} impl std::fmt::Debug for S {} impl app::Shown for S {}
