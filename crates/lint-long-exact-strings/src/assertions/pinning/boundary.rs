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
    const SPECIFIC: [(Scan, Mark, Scan); 10] = [
        (Scan::Questioned, Mark::Close, Scan::Debug),
        (Scan::Text, Mark::Open, Scan::Opened),
        (Scan::Opened, Mark::Close, Scan::Text),
        (Scan::Named, Mark::Close, Scan::Text),
        (Scan::Spec, Mark::Close, Scan::Text),
        (Scan::Opened, Mark::Open, Scan::Text),
        (Scan::Opened, Mark::Spec, Scan::Spec),
        (Scan::Named, Mark::Spec, Scan::Spec),
        (Scan::Spec, Mark::Debug, Scan::Questioned),
        (Scan::Questioned, Mark::Debug, Scan::Questioned),
    ];
    const OTHERWISE: [(Scan, Scan); 6] = [
        (Scan::Debug, Scan::Debug),
        (Scan::Text, Scan::Text),
        (Scan::Opened, Scan::Named),
        (Scan::Named, Scan::Named),
        (Scan::Spec, Scan::Spec),
        (Scan::Questioned, Scan::Spec),
    ];

    pub fn next(self, letter: char) -> Scan {
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
    const NAMED: [(char, Mark); 4] = [('{', Mark::Open), ('}', Mark::Close), (':', Mark::Spec), ('?', Mark::Debug)];
}

impl From<char> for Mark {
    fn from(letter: char) -> Mark {
        Mark::NAMED.iter().find(|(named, _)| *named == letter).map_or(Mark::Other, |(_, mark)| *mark)
    }
}
