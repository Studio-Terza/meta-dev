use text_lines::Lines;

#[derive(Debug)]
pub struct LabelSet(String);

impl LabelSet {
    pub(crate) fn listed(names: String) -> LabelSet {
        LabelSet(names)
    }

    pub(crate) fn carrying(&self, label: &str) -> Option<&str> {
        Lines::from(self.0.as_str()).find(|name| *name == label)
    }
}
