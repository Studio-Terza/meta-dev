use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Authorship {
    Owner,
    Lead,
    Bystander,
}

impl fmt::Display for Authorship {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Authorship::Owner => formatter.write_str("owner"),
            Authorship::Lead => formatter.write_str("lead"),
            Authorship::Bystander => formatter.write_str("bystander"),
        }
    }
}
