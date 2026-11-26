use strum::Display;

#[derive(Display, Clone, Copy, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Document {
    Limits,
    Measured,
}
