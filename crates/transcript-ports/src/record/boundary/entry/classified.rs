use crate::Line;

pub trait Classified: Sized {
    fn line(self) -> Line;

    fn heard(self) -> Option<Line> {
        Some(self.line())
    }
}
