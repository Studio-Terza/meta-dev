pub enum Door {
    Open,
    Shut,
    Ajar,
}

pub struct Doors;

impl Doors {
    pub const EVERY: [Door; 3] = [Door::Open, Door::Shut, Door::Ajar];
    pub const CLOSABLE: [Door; 2] = [Door::Open, Door::Ajar];
    pub const SPELLED: [(&'static str, Door); 1] = [("open", Door::Open)];

    pub fn spelling(door: Door) -> &'static str {
        match door {
            Door::Open => "open",
            Door::Shut => "shut",
            Door::Ajar => "ajar",
        }
    }
}
