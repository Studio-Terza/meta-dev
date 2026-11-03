pub struct Raw(Text);
pub struct Policy(Text);
pub struct Text;

impl From<&Raw> for Policy {
    fn from(raw: &Raw) -> Policy {
        Policy(Text)
    }
}

impl Raw {
    pub fn policy(&self) -> Policy {
        Policy::from(self)
    }

    pub fn read(&self) -> Policy {
        Policy(Text)
    }
}
