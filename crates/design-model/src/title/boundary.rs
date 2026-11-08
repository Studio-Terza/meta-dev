use grammar::notes::Line;

use super::Title;

impl From<&Line> for Title {
    fn from(line: &Line) -> Title {
        Title(line.to_string())
    }
}
