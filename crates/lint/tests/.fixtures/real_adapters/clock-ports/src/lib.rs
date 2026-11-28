#[port_core::port]
pub trait Clock {}

pub trait Files {}

#[derive(Debug)]
pub struct SystemClock;

impl port_core::Real for SystemClock {}

#[derive(Debug)]
pub struct Fake;

impl Clock for Fake {}
