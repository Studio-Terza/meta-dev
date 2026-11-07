#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
    Text,
    Opened,
    Named,
    Spec,
    Questioned,
    Debug,
}

impl Scan {
    const SPECIFIC: [(Self, Mark, Self); 10] = [
        (Self::Questioned, Mark::Close, Self::Debug),
        (Self::Text, Mark::Open, Self::Opened),
        (Self::Opened, Mark::Close, Self::Text),
        (Self::Named, Mark::Close, Self::Text),
        (Self::Spec, Mark::Close, Self::Text),
        (Self::Opened, Mark::Open, Self::Text),
        (Self::Opened, Mark::Spec, Self::Spec),
        (Self::Named, Mark::Spec, Self::Spec),
        (Self::Spec, Mark::Debug, Self::Questioned),
        (Self::Questioned, Mark::Debug, Self::Questioned),
    ];
    const OTHERWISE: [(Self, Self); 6] = [
        (Self::Debug, Self::Debug),
        (Self::Text, Self::Text),
        (Self::Opened, Self::Named),
        (Self::Named, Self::Named),
        (Self::Spec, Self::Spec),
        (Self::Questioned, Self::Spec),
    ];

    pub fn next(self, letter: char) -> Self {
        let mark = Mark::from(letter);
        Self::SPECIFIC
            .iter()
            .filter(|(from, _, _)| *from == self)
            .find(|(_, on, _)| *on == mark)
            .map(|(_, _, to)| *to)
            .or_else(|| Self::OTHERWISE.iter().find(|(from, _)| *from == self).map(|(_, to)| *to))
            .unwrap_or(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Open,
    Close,
    Spec,
    Debug,
    Other,
}

impl Mark {
    const NAMED: [(char, Self); 4] = [('{', Self::Open), ('}', Self::Close), (':', Self::Spec), ('?', Self::Debug)];
}

impl From<char> for Mark {
    fn from(letter: char) -> Self {
        Self::NAMED.iter().find(|(named, _)| *named == letter).map_or(Self::Other, |(_, mark)| *mark)
    }
}
