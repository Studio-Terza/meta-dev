use std::convert::Infallible;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinning {
    LineBreak,
    Debug,
    Plain,
}

impl FromStr for Pinning {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Pinning, Infallible> {
        let placeholders = text.replace("{{", "");
        let debug = placeholders
            .split('{')
            .skip(1)
            .filter_map(|part| part.split_once('}'))
            .any(|(placeholder, _)| placeholder.rsplit_once(':').is_some_and(|(_, spec)| spec.ends_with('?')));
        Ok(match (text.contains('\n'), debug) {
            (true, _) => Pinning::LineBreak,
            (false, true) => Pinning::Debug,
            (false, false) => Pinning::Plain,
        })
    }
}
