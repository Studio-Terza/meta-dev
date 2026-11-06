use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Authorship {
    Owner,
    Lead,
    Bystander,
}

impl fmt::Display for Authorship {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Authorship::Owner => "owner",
            Authorship::Lead => "lead",
            Authorship::Bystander => "bystander",
        })
    }
}
