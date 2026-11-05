use text_lines::Lines;

struct Sample;

impl Sample {
    fn bracketed(text: &&str) -> String {
        Lines::from(*text).map(|line| ["<", line, ">"].concat()).collect()
    }
}

laws::table!(
    text_splits_at_each_line_break_without_keeping_it,
    Sample::bracketed,
    [
        "" => "",
        "a" => "<a>",
        "a\nb" => "<a><b>",
        "a\nb\n" => "<a><b>",
        "a\n\nb" => "<a><><b>",
        "\n" => "<>",
        "\n\n" => "<><>",
        "\na" => "<><a>",
        "é\nü" => "<é><ü>",
        "a b\tc\nd" => "<a b\tc><d>",
        "a\r\nb" => "<a\r><b>",
    ]
);
