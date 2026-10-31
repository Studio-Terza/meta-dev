#![allow(clippy::disallowed_methods)]

/// A test writes a command line as one string; this is the one place it becomes its words.
pub fn words(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::words;

    #[test]
    fn a_command_line_is_its_whitespace_separated_words() {
        assert_eq!(words("claim  task-1\t--self"), ["claim", "task-1", "--self"]);
        assert_eq!(words(""), Vec::<String>::new());
    }
}
