use port_core::port;

#[port]
pub trait Store {
    fn read(&self);
}

#[derive(Debug)]
pub struct Disk;

impl StoreFiller for Disk {
    fn read(&self) {
        std::fs::read("store").ok();
    }
}

impl Disk {
    pub fn open() {
        std::fs::read("store").ok();
    }
}
