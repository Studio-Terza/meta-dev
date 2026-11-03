use std::string::FromUtf8Error;

use file_ports::Text;

use super::Excerpt;

impl From<FromUtf8Error> for Excerpt {
    fn from(failure: FromUtf8Error) -> Excerpt {
        Excerpt(failure.utf8_error().to_string())
    }
}

impl From<&Text> for Excerpt {
    fn from(reason: &Text) -> Excerpt {
        Excerpt(reason.to_string())
    }
}
