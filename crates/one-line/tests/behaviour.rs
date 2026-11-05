use one_line::OneLine;

struct Read;

impl Read {
    fn line(text: &&str) -> Result<String, ()> {
        OneLine::try_from((*text).to_owned()).map(|line| line.to_string()).map_err(|_| ())
    }

    fn deserialized(json: &&str) -> Result<String, ()> {
        serde_json::from_str::<OneLine>(json).map(|line| line.as_ref().to_owned()).map_err(|_| ())
    }
}

laws::table!(
    text_with_a_line_break_or_control_character_is_not_one_line,
    Read::line,
    [
        "" => Ok(String::new()),
        "a b" => Ok("a b".to_owned()),
        "é ü" => Ok("é ü".to_owned()),
        "a\nb" => Err(()),
        "a\rb" => Err(()),
        "a\tb" => Err(()),
        "a\u{7f}" => Err(()),
        "\u{0}" => Err(()),
    ]
);

laws::table!(
    a_line_read_from_json_is_checked,
    Read::deserialized,
    [
        "\"a b\"" => Ok("a b".to_owned()),
        "\"a\\nb\"" => Err(()),
        "1" => Err(()),
    ]
);
