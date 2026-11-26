use std::fmt;

use design_store::StopRow;

#[derive(Debug)]
pub struct Trail(Vec<StopRow>);

impl From<Vec<StopRow>> for Trail {
    fn from(rows: Vec<StopRow>) -> Trail {
        Trail(rows)
    }
}

impl fmt::Display for Trail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "session\tshells\tagents\thalt\tread\tquoted\tkept\tfate\ttokens\tmillis")?;
        self.0.iter().try_for_each(|row| writeln!(formatter, "{row}"))
    }
}
