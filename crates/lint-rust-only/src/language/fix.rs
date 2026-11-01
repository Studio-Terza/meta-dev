use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    Rewrite,
    MoveToAssets,
}

impl fmt::Display for Fix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Fix::Rewrite => "rewrite it in Rust",
            Fix::MoveToAssets => {
                "if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust"
            }
        })
    }
}
