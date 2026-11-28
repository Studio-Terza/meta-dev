#[derive(Debug)]
pub struct SystemClock;

impl port_core::Real for SystemClock {}

#[derive(Debug)]
pub struct Fake;

impl Clock for Fake {}
