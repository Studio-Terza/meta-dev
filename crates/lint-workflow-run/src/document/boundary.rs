use saphyr::{LoadableYamlNode, MarkedYaml, ScanError};

use super::Document;

impl<'text> Document<'text> {
    pub(crate) fn loaded(text: &'text str) -> Result<Document<'text>, ScanError> {
        MarkedYaml::load_from_str(text).map(Document)
    }
}
