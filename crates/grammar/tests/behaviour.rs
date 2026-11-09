#![deny(clippy::wildcard_enum_match_arm)]

use grammar::anchor::Anchoring;
use grammar::artifact::Artifacts;
use grammar::cited::Cited;
use grammar::flag::Flags;
use grammar::ident::Words;
use grammar::line::Unbroken;
use grammar::lstart::Stamp;
use grammar::message::{Depth, Lexeme, Lexemes, Mark, Number};
use grammar::node::{Ids, Spelled};
use grammar::notes::{Ending, Entry, Line, Notes, Quote, Reading, Unclosed};
use grammar::output::field::{Bare, Id, Integer, Quoted};
use grammar::output::{Fields, Unread};
use grammar::reply::Inked;
use grammar::search::Term;
use grammar::search::Terms;
use grammar::slug::Letters;
use laws::proptest::collection::vec;
use laws::proptest::prelude::{Strategy, prop_assert_eq, prop_oneof};
use laws::proptest::sample::select;

struct Marks;

impl Marks {
    fn of(text: &str) -> Vec<Mark> {
        Lexemes::scanned(&text).into_iter().map(|lexeme| lexeme.mark()).filter(|mark| *mark != Mark::Plain).collect()
    }
}

laws::table!(
    markers_are_recognised_at_line_starts,
    Marks::of,
    [
        "1. one\n2) two" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Top)],
        "1. one\n  2. sub" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Nested)],
        "**1. one**\n**2.** two" => vec![Mark::Numbered(Number::new(1), Depth::Top), Mark::Numbered(Number::new(2), Depth::Top)],
        "- a\n* b\n> c" => vec![Mark::Bulleted, Mark::Bulleted, Mark::Quoted],
        "yes. no? ok" => vec![Mark::Stop, Mark::Stop],
        "v0.2.1 is fine" => Vec::<Mark>::new(),
        "see 85b287b3-4d5e-4f60-8a9b-0c1d2e3f4a5b now" => vec![Mark::Uuid],
        "line\nnext" => vec![Mark::Break],
    ]
);

laws::law!(lexemes_tile_the_text_without_gaps, "[ -~\n\t•é]{0,60}", text, {
    let rebuilt: String = Lexemes::scanned(&text).iter().map(|lexeme| lexeme.span().slice(&text)).collect();
    prop_assert_eq!(rebuilt, text);
});

laws::table!(
    slugs_keep_letters_and_digits_and_replace_every_other_character,
    Letters::slugged,
    [
        "/Users/me/my_repo" => "-Users-me-my-repo",
        "abc123" => "abc123",
        "a.b é" => "a-b--",
        "" => "",
    ]
);

laws::table!(
    camel_names_split_into_lowered_humps,
    Words::lowered_humps,
    [
        "Transcripts" => vec!["transcripts"],
        "ReadFrom" => vec!["read", "from"],
        "lowerThenUpper" => vec!["lower", "then", "upper"],
        "" => Vec::<String>::new(),
    ]
);

laws::table!(
    snake_names_raise_into_camel_case,
    Words::camel,
    [
        "read_from" => "ReadFrom",
        "sessions" => "Sessions",
        "a__b_" => "AB",
        "" => "",
    ]
);

struct Lexed;

impl Lexed {
    fn terms(words: &[&str]) -> Vec<String> {
        Terms::lexed(words.iter().copied())
            .map(|terms| [terms.first()].into_iter().chain(terms.rest()).map(ToString::to_string).collect())
            .unwrap_or_default()
    }
}

laws::table!(
    search_words_lex_into_lowered_terms_split_at_every_other_character,
    Lexed::terms,
    [
        ["Pinning-tests", "NOW"] => vec!["pinning", "tests", "now"],
        ["\"a OR b*\"", "c:d"] => vec!["a", "or", "b", "c", "d"],
        ["Écoute", "v0.2"] => vec!["écoute", "v0", "2"],
        ["--", "..."] => Vec::<String>::new(),
    ]
);

laws::table!(
    node_ids_are_n_and_a_number_without_leading_zeros_that_fits,
    Ids::spelled,
    [
        "n0" => Spelled::Numbered(0),
        "n42" => Spelled::Numbered(42),
        "n9223372036854775807" => Spelled::Numbered(i64::MAX),
        "n00" => Spelled::Misnumbered,
        "n001" => Spelled::Misnumbered,
        "n9223372036854775808" => Spelled::Misnumbered,
        "n99999999999999999999" => Spelled::Misnumbered,
        "n" => Spelled::Unspelled,
        "42" => Spelled::Unspelled,
        "n42x" => Spelled::Unspelled,
    ]
);

struct Spellings;

impl Spellings {
    fn artifact(text: &str) -> Option<[String; 2]> {
        Artifacts::spelled(text).map(|spelled| spelled.joined(|kind, name| [kind.to_owned(), name.to_owned()]))
    }
}

laws::table!(
    an_artifact_is_spelled_as_a_lowercase_kind_a_colon_and_a_name,
    |text: &&str| Spellings::artifact(text),
    [
        "lint:no-comments" => Some(["lint".to_owned(), "no-comments".to_owned()]),
        "behaviour:tests::a_law" => Some(["behaviour".to_owned(), "tests::a_law".to_owned()]),
        "tool:crates/design.rs" => Some(["tool".to_owned(), "crates/design.rs".to_owned()]),
        "Lint:x" => None,
        "lint" => None,
        "lint:" => None,
        ":x" => None,
        "lint:x y" => None,
        "lint :x" => None,
        "lint: x" => None,
        "lint:-x" => None,
        "" => None,
    ]
);

laws::table!(
    one_unbroken_line_with_a_visible_character_is_spelled_whole,
    |text: &&str| Unbroken::spelled(text).map(str::to_owned),
    [
        "a condition" => Some("a condition".to_owned()),
        "  padded " => Some("  padded ".to_owned()),
        "\tx" => Some("\tx".to_owned()),
        "" => None,
        "  " => None,
        "\t" => None,
        "a\nb" => None,
        "a\r\n" => None,
        "\na" => None,
        "\u{b}" => None,
        "\u{c}" => None,
        "\u{a0}" => None,
        "\u{2028}" => None,
        "\u{85}" => None,
        "\u{3000}\t" => None,
        "\u{a0}x" => Some("\u{a0}x".to_owned()),
        "x\u{b}y" => None,
        "x\u{c}y" => None,
        "x\u{85}y" => None,
        "x\u{2028}y" => None,
        "x\u{2029}y" => None,
        "x\ry" => None,
        "x\u{2029}" => None,
        "\u{85}x" => None,
        "x\u{2007}y" => Some("x\u{2007}y".to_owned()),
    ]
);

laws::table!(
    a_word_led_by_two_dashes_is_spelled_as_a_flag,
    |text: &&str| Flags::spelled(text).map(str::to_owned),
    [
        "--reason" => Some("--reason".to_owned()),
        "--" => Some("--".to_owned()),
        "---x" => Some("---x".to_owned()),
        "--no such" => Some("--no such".to_owned()),
        "-r" => None,
        "demo" => None,
        "a--b" => None,
        " --reason" => None,
        "" => None,
    ]
);

struct Read;

#[derive(Debug, PartialEq, Eq)]
struct Read6 {
    node: i64,
    quoted: Term,
    bare: Term,
    integer: u64,
    ordinal: usize,
    message: Term,
}

impl Read {
    fn quoted(text: &str) -> Option<Term> {
        let mut fields = Fields::lexed(&Fields::quoting(text)).ok()?;
        let read = fields.field::<Quoted<Term>>().ok()?;
        fields.ended(read).ok()
    }

    fn line(text: &&str) -> Option<Read6> {
        let mut fields = Fields::lexed(text).ok()?;
        let line = Read::fields(&mut fields).ok()?;
        fields.ended(line).ok()
    }

    fn fields(fields: &mut Fields) -> Result<Read6, Unread> {
        Ok(Read6 {
            node: fields.field::<Id<Node>>()?.number(),
            quoted: fields.field::<Quoted<Term>>()?,
            bare: fields.field::<Bare<Term>>()?,
            integer: fields.past(&["to".to_owned()])?.field::<Integer<u64>>()?,
            ordinal: fields.field::<Cited>()?.ordinal(),
            message: fields.field::<Bare<Term>>()?,
        })
    }
}

struct Node(i64);

impl std::str::FromStr for Node {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Node, Self::Err> {
        text.get(1..).unwrap_or_default().parse().map(Node)
    }
}

impl Node {
    fn number(self) -> i64 {
        self.0
    }
}

laws::roundtrip!(
    a_spec_line_without_trailing_space_is_parsed_and_shown_back_as_written,
    "repair: main|spec: ( ?[!-~]{1,8})( [!-~]{1,8}){0,3}",
    |text: &String| text.parse::<grammar::spec_line::Line>().ok(),
    |line: &Option<grammar::spec_line::Line>| line.as_ref().map(ToString::to_string),
    Some
);

laws::roundtrip!(
    any_text_is_quoted_and_lexed_back_as_one_quoted_field,
    "(?s).{0,40}",
    |text: &String| text.clone(),
    |text: &String| Read::quoted(text),
    |text: String| Some(Term::new(text))
);

laws::table!(
    an_output_line_lexes_into_words_refs_ids_quotes_and_numbers,
    Read::line,
    [
        "n4 \"de\\\"mo\" feature to 12 deadbeef.3 85b287b3-4d5e-4f60\n" => Some(Read6 {
            node: 4,
            quoted: Term::new("de\"mo".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 12,
            ordinal: 3,
            message: Term::new("85b287b3-4d5e-4f60".to_owned()),
        }),
        "n4 \"x\" feature to 1234567 1a2b3c4.0 0a1b2c3" => Some(Read6 {
            node: 4,
            quoted: Term::new("x".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 1_234_567,
            ordinal: 0,
            message: Term::new("0a1b2c3".to_owned()),
        }),
        "n4 \"x\" feature to 7 9abcdef0.12 7e57ab1-0000" => Some(Read6 {
            node: 4,
            quoted: Term::new("x".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 7,
            ordinal: 12,
            message: Term::new("7e57ab1-0000".to_owned()),
        }),
        "n4 \"x\" feature to 7 o1.0 O1" => Some(Read6 {
            node: 4,
            quoted: Term::new("x".to_owned()),
            bare: Term::new("feature".to_owned()),
            integer: 7,
            ordinal: 0,
            message: Term::new("O1".to_owned()),
        }),
        "n4 \"x\" feature to 7 o1. o1" => None,
        "n4 \"x\" feature to 7 o1.01 o1" => None,
        "n4 \"x\" feature to 1a2b3c4 1a2b3c4.0 1a2b3c4" => None,
        "n4 \"x\" feature to 12 deadbeef.3 deadbeef extra" => None,
        "n4 \"x\" feature from 12 deadbeef.3 deadbeef" => None,
        "n4 \"x\" ( feature" => None,
    ]
);

struct Inks;

#[derive(Debug, PartialEq, Eq)]
struct Anchored {
    message: String,
    ordinal: Option<usize>,
}

impl Inks {
    fn of(text: &&str) -> String {
        Lexemes::scanned(text).iter().filter_map(Lexeme::ink).map(|ink| ink.slice(text)).collect()
    }

    fn anchored(text: &&str) -> Option<Anchored> {
        Anchoring::read(text)
            .map(|anchoring| Anchored { message: anchoring.message().to_owned(), ordinal: anchoring.ordinal() })
    }
}

laws::table!(
    review_a_lexeme_ink_sheds_unicode_whitespace,
    Inks::of,
    [
        "agreed.\u{a0}" => "agreed.".to_owned(),
        "\u{3000}fine" => "fine".to_owned(),
        "\u{2003}one\u{85}two\u{2029}" => "onetwo".to_owned(),
        "\u{200b}kept" => "\u{200b}kept".to_owned(),
    ]
);

laws::table!(
    an_anchor_reads_as_a_message_id_or_one_with_a_segment_index,
    Inks::anchored,
    [
        "85b287b3-4d5e" => Some(Anchored { message: "85b287b3-4d5e".to_owned(), ordinal: None }),
        "85b287b3-4d5e#0" => Some(Anchored { message: "85b287b3-4d5e".to_owned(), ordinal: Some(0) }),
        "o1#12" => Some(Anchored { message: "o1".to_owned(), ordinal: Some(12) }),
        "o1#" => None,
        "o1#01" => None,
        "o1#2#3" => None,
        "#2" => None,
        "" => None,
    ]
);

laws::table!(
    a_process_start_time_is_read_as_two_words_and_five_numbers,
    |text: &&'static str| Stamp::read(text).map(|stamp| (stamp.words(), stamp.numbers())),
    [
        "Fri Nov  6 01:16:00 2026    \n" => Some((["Fri", "Nov"], [6, 1, 16, 0, 2026])),
        "Wed Sep 30 22:11:39 2026" => Some((["Wed", "Sep"], [30, 22, 11, 39, 2026])),
        "Fri Nov  6 01:16 2026" => None,
        "Fri Nov  6 01:16:00" => None,
        "Fri Nov  6 01:16:00 2026 2027" => None,
        "Fri Nov  6 01:16 00:2026" => None,
        "Fri Nov  6 01:16:00 Sat" => None,
        "Fri 11  6 01:16:00 2026" => None,
        "fri Nov  6 01:16:00 2026" => None,
        "Fri Nov  6 01:16:00 99999" => None,
        "" => None,
    ]
);

laws::table!(
    a_reply_is_text_with_ink_and_no_nul,
    |text: &&'static str| Inked::read(text).map(|inked| inked.as_ref().len()),
    [
        "place(a.0, n1, about)" => Some(21),
        "  x  " => Some(5),
        "x\ny" => Some(3),
        "" => None,
        " \n\t" => None,
        "x\0y" => None,
        "\0" => None,
    ]
);

const NOTED: &str = "(- |-|\\* |\\*|\\*\\*|\\+ |\\+|# |#|RULING|LEAD|[a-z]{1,4}n't|: |:|\\(|\\)|\"[a-z: ]{0,6}\"|'[a-z:' ]{0,6}'|\"|'|[0-9a-f]{8}|[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|2026-11-0[1-9]|2026-11-0[1-9][Tt ][0-9]{2}:[0-9]{2}(:[0-9]{2}(\\.[0-9]{1,4})?)?([Zz]|[+-][0-9]{2}:[0-9]{2})?|[a-zé]{1,5}'s|[a-z]{1,5}s'|'[a-z]{1,5}s' [a-z]{1,5}'|\"[a-z]{1,5}s' [a-z]{1,5}\"|[a-z]{1,6}| |\t|\r|[.,;~é]){0,14}";

const INKED: &str =
    "(- |# |RULING|: |\\(|\\)|\"[a-z ]{0,6}\"|'[a-z ]{0,6}'|[0-9a-f]{8}|2026-11-0[1-9]|[a-z]{1,6}| |[.,;]){0,10}[a-z.]";

const LAID: &str = "[ \t]{0,3}\r?";

laws::law!(a_notes_line_is_rendered_back_to_its_exact_text, NOTED, text, {
    prop_assert_eq!(Line::read(1, &text).to_string(), text);
});

laws::law!(a_notes_file_is_rendered_back_to_its_exact_text, (vec(NOTED, 0..8), "\r?\n", "(\r?\n)?"), file, {
    let (lines, separator, ending) = file;
    let text = [lines.join(&separator), ending].concat();
    prop_assert_eq!(Notes::read(&text).to_string(), text);
});

laws::law!(
    a_double_quoted_quote_keeps_every_apostrophe_inside_it,
    "'?[a-z]{1,6}('s|s'|n't|')?( '?[a-z]{1,6}('s|s'|n't|')?){0,3}",
    quote,
    {
        let line = format!("RULING (owner): \"{quote}\", as said");
        prop_assert_eq!(Line::read(1, &line).read_by(Named), Ok([vec![quote], vec![], vec![]]));
    }
);

laws::law!(
    a_double_quote_keeps_its_contractions_and_possessives_and_the_line_keeps_the_kind_its_opening_word_gives,
    (
        select(vec![
            ("RULING ", "ruling"),
            ("- RULING ", "ruling"),
            ("* RULING ", "ruling"),
            ("+ **RULING** ", "ruling"),
            ("- LEAD ", "lead"),
            ("**LEAD** ", "lead"),
            ("Lead ", "remark"),
        ]),
        "[a-z]{1,6}('s|s'|n't)?( '?[a-z]{1,6}('s|s'|n't)?'?){0,3}",
        "( '?[a-z]{1,6}('s|s'|n't)?'?){0,4}",
    ),
    said,
    {
        let ((opening, kind), quote, after) = said;
        let line = Line::read(1, &format!("{opening}(owner, 0a1b2c3d): \"{quote}\"{after}"));
        prop_assert_eq!(line.read_by(Shaped), kind);
        prop_assert_eq!(line.read_by(Named), Ok([vec![quote], vec!["0a1b2c3d".to_owned()], vec![]]));
    }
);

const CLOCK: &str = "([Tt ][0-2][0-9]:[0-5][0-9](:[0-5][0-9](\\.[0-9]{1,9})?)?([Zz]|[+-][0-2][0-9]:[0-5][0-9])?)?";

laws::law!(
    a_date_stamped_with_any_rfc_3339_time_offset_or_fraction_leaves_the_line_the_kind_its_first_word_gives,
    (
        select(vec!["", "- "]),
        "2026-11-0[1-9]",
        CLOCK,
        select(vec![("RULING", "ruling"), ("LEAD", "lead"), ("plan", "remark")])
    ),
    said,
    {
        let (bullet, day, clock, (word, kind)) = said;
        let text = format!("{bullet}{day}{clock} {word} (owner): \"yes\"");
        let line = Line::read(1, &text);
        prop_assert_eq!(line.read_by(Shaped), kind);
        prop_assert_eq!(line.read_by(Named), Ok([vec!["yes".to_owned()], vec![], vec![day]]));
        prop_assert_eq!(line.to_string(), text);
    }
);

const MARKUP: [&str; 24] = [
    "- ",
    "* ",
    "+ ",
    "-",
    "**",
    "*",
    "_",
    "__",
    "1. ",
    "12) ",
    "3.",
    "> ",
    ">",
    "[ ] ",
    "[x] ",
    "[X]",
    "2026-11-05",
    "2026-11-05: ",
    "2026-11-05 04:07 ",
    "2026-11-05T04:07",
    "2026-11-05T04:07:00Z: ",
    ": ",
    ":",
    "\t",
];

laws::law!(
    any_allowed_markup_before_the_first_word_leaves_the_line_the_kind_that_word_gives,
    (
        vec(select(MARKUP.to_vec()), 0..8),
        select(vec![
            ("RULING", "ruling"),
            ("LEAD", "lead"),
            ("plan", "remark"),
            ("Ruling", "remark"),
            ("RULINGS", "remark")
        ]),
    ),
    said,
    {
        let (markup, (word, kind)) = said;
        let line = Line::read(1, &format!("{} {word} (owner): x", markup.concat()));
        prop_assert_eq!(line.read_by(Shaped), kind);
    }
);

laws::law!(
    any_other_opener_before_ruling_makes_the_line_a_remark,
    (
        vec(select(MARKUP.to_vec()), 0..4),
        select(vec!["\"", "~~", "~", "|", "`", "(", "{", ".", ",", "#", "<", "0a1b2c3d", "note"]),
        vec(select(MARKUP.to_vec()), 0..4),
    ),
    said,
    {
        let (before, opener, after) = said;
        let line = Line::read(1, &format!("{} {opener}{} RULING (owner): \"yes\"", before.concat(), after.concat()));
        prop_assert_eq!(line.read_by(Shaped), "remark");
    }
);

laws::law!(
    a_bare_number_before_ruling_makes_the_line_a_remark,
    (vec(select(MARKUP.to_vec()), 0..4), "[0-9]{1,12}", vec(select(MARKUP.to_vec()), 0..4)),
    said,
    {
        let (before, number, after) = said;
        let line = Line::read(1, &format!("{} {number} {} RULING (owner): \"yes\"", before.concat(), after.concat()));
        prop_assert_eq!(line.read_by(Shaped), "remark");
    }
);

laws::law!(
    a_single_quote_names_no_quote_and_hides_no_cite_or_date,
    ("( '?[a-z]{1,6}('s|s'|n't)?'?| '){0,5}", "( '?[a-z]{1,6}('s|s'|n't)?'?| '){0,5}",),
    said,
    {
        let (before, after) = said;
        let line = Line::read(1, &format!("RULING:{before} '0a1b2c3d' on '2026-11-05'{after}"));
        prop_assert_eq!(line.read_by(Shaped), "ruling");
        prop_assert_eq!(line.read_by(Named), Ok([vec![], vec!["0a1b2c3d".to_owned()], vec!["2026-11-05".to_owned()]]));
    }
);

laws::law!(
    a_cite_before_a_possessive_is_named,
    vec(
        (
            "[a-z]{1,6}('s|n't)?",
            "[0-9a-f]{8}|[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
            "('s|'|:|,|\\.|\\))?",
        ),
        0..5
    ),
    said,
    {
        let words: String = said.iter().flat_map(|(word, cite, after)| [" ", word, " ", cite, after]).collect();
        let cites: Vec<String> = said.into_iter().map(|(_word, cite, _after)| cite).collect();
        let line = Line::read(1, &format!("RULING (owner):{words}"));
        prop_assert_eq!(line.read_by(Shaped), "ruling");
        prop_assert_eq!(line.read_by(Named), Ok([vec![], cites, vec![]]));
    }
);

laws::law!(
    the_content_lines_of_a_notes_file_gathered_by_number_give_the_file_back_with_layout_lines_empty,
    (
        vec(
            prop_oneof![INKED.prop_map(|text| (text.clone(), text)), LAID.prop_map(|text| (text, String::new())),],
            0..8
        ),
        select(vec![(Ending::Break, "\n"), (Ending::Open, "")]),
    ),
    file,
    {
        let (lines, (ending, ended)) = file;
        let lines: Vec<(String, String)> = lines.into_iter().chain([("x".to_owned(), "x".to_owned())]).collect();
        let text = [lines.iter().map(|(line, _laid)| line.as_str()).collect::<Vec<&str>>().join("\n").as_str(), ended]
            .concat();
        let laid = [lines.iter().map(|(_line, laid)| laid.as_str()).collect::<Vec<&str>>().join("\n").as_str(), ended]
            .concat();
        let notes = Notes::read(&text);
        prop_assert_eq!(notes.ending(), ending);
        let content = notes
            .lines()
            .iter()
            .filter(|line| line.read_by(Shaped) != "layout")
            .map(|line| Line::read(line.number(), &line.to_string()))
            .collect::<Vec<Line>>();
        prop_assert_eq!(Notes::gathered(content, ending).to_string(), laid);
    }
);

#[derive(Clone, Copy)]
struct Shaped;

impl Reading for Shaped {
    type Output = &'static str;

    fn layout(self, _line: &Line) -> &'static str {
        "layout"
    }

    fn heading(self, _line: &Line, _entry: &Entry) -> &'static str {
        "heading"
    }

    fn ruling(self, _line: &Line, _entry: &Entry) -> &'static str {
        "ruling"
    }

    fn lead(self, _line: &Line, _entry: &Entry) -> &'static str {
        "lead"
    }

    fn remark(self, _line: &Line, _entry: &Entry) -> &'static str {
        "remark"
    }
}

#[derive(Clone, Copy)]
struct Named;

type Said = Result<[Vec<String>; 3], Unclosed>;

impl Named {
    fn words(entry: &Entry) -> Said {
        let quotes: Result<Vec<String>, Unclosed> =
            entry.quotes().into_iter().map(|quote| quote.map(ToString::to_string).map_err(Clone::clone)).collect();
        quotes.map(|quotes| {
            [
                quotes,
                entry.cites().iter().map(ToString::to_string).collect(),
                entry.dates().iter().map(ToString::to_string).collect(),
            ]
        })
    }
}

impl Reading for Named {
    type Output = Said;

    fn layout(self, _line: &Line) -> Said {
        Ok(<[Vec<String>; 3]>::default())
    }

    fn heading(self, _line: &Line, entry: &Entry) -> Said {
        Named::words(entry)
    }

    fn ruling(self, _line: &Line, entry: &Entry) -> Said {
        Named::words(entry)
    }

    fn lead(self, _line: &Line, entry: &Entry) -> Said {
        Named::words(entry)
    }

    fn remark(self, _line: &Line, entry: &Entry) -> Said {
        Named::words(entry)
    }
}

laws::table!(
    a_notes_line_is_layout_a_heading_or_an_entry_whose_kind_is_its_first_alphabetic_word,
    |text: &&str| Line::read(1, text).read_by(Shaped),
    [
        "" => "layout",
        "  \t" => "layout",
        " " => "layout",
        "\t" => "layout",
        " x" => "remark",
        "\t# Notes" => "remark",
        "# Notes" => "heading",
        "## Later notes (RULING)" => "heading",
        "#443 merged" => "remark",
        "- 2026-11-06 lead: x" => "remark",
        "-x" => "remark",
        "plain prose" => "remark",
        "RULING (owner, explicit, 2026-11-05): \"the new trellis is approved.\"" => "ruling",
        "- 2026-11-06 RULING (owner, 0a1b2c3d): 'every bed gets a label'" => "ruling",
        "RULING (owner, explicit, 2026-11-05: \"done\"): the measure" => "ruling",
        "- RULING 2026-11-07 (0f1e2d3c): \"names may change\"" => "ruling",
        "Lead: the RULING above stands" => "remark",
        "(Owner moved on with \"next\": RULING aside)" => "remark",
        "RULINGS (owner): plural" => "remark",
        "RULING's scope (owner): possessive" => "remark",
        "RULING (owner): as c3c3c3c3's reply says" => "ruling",
        "(RULING: aside) and more" => "remark",
        "(owner) RULING: approved" => "remark",
        "(a (b) c) RULING: nested" => "remark",
        "a) RULING: unbalanced" => "remark",
        "RULING" => "ruling",
        "- LEAD 2026-11-08: the beds are dug" => "lead",
        "- 2026-11-08 LEAD (east bed): dug" => "lead",
        "- LEAD 2026-11-08 review (an imported RULING kept apart): the beds are dug" => "lead",
        "LEADS (the rows): plural" => "remark",
        "Lead: the beds are dug" => "remark",
        "- 2026-11-08 lead: the beds are dug" => "remark",
        " RULING (owner): indented" => "ruling",
        "- 2026-11-06 RULING (owner, 'yes'): we don't change Ann's plan" => "ruling",
        "2026-11-05T04:07+01:00 RULING (owner): x" => "ruling",
        "- 2026-11-05T04:07:09.125-03:30 LEAD: the beds are dug" => "lead",
        "2026-11-05 04:07:09.5Z RULING (owner): x" => "ruling",
        "* RULING (owner): starred" => "ruling",
        "+ RULING (owner): plussed" => "ruling",
        "**RULING** (owner): bold" => "ruling",
        "*RULING*: leaning" => "ruling",
        "2026-11-05: RULING (owner): dated" => "ruling",
        "- 2026-11-05: **RULING** (owner): every prefix" => "ruling",
        "+ **LEAD** 2026-11-08: the beds are dug" => "lead",
        ": RULING (owner): a bare colon" => "ruling",
        "2026-11-05 : RULING (owner): a spaced colon" => "ruling",
        "1. RULING (owner): numbered" => "ruling",
        "> RULING (owner): quoted" => "ruling",
        "**2026-11-05**: RULING (owner): a bold date" => "ruling",
        "- [x] RULING (owner): a task" => "ruling",
        "- [ ] LEAD: an open task" => "lead",
        "- [x] done: RULING aside" => "remark",
        "__RULING__ (owner): underlined" => "ruling",
        "_LEAD_: the beds are dug" => "lead",
        "12) RULING (owner): numbered" => "ruling",
        "3 RULING rows still lack labels." => "remark",
        "- 3 RULING rows still lack labels." => "remark",
        "1234567 RULING (owner): seven digits" => "remark",
        "12345678 RULING (owner): eight digits" => "remark",
        "1234567. RULING (owner): numbered" => "ruling",
        "2026-11-05 04:07 RULING (owner): timed" => "ruling",
        "2026-11-05T04:07 RULING (owner): timed" => "ruling",
        "- 2026-11-05T04:07:00Z: RULING (owner): timed" => "ruling",
        "04:07 RULING (owner): a bare time" => "remark",
        "2026-11 RULING (owner): a bare month" => "remark",
        "\"RULING\" lines must quote the owner." => "remark",
        "| RULING | count |" => "remark",
        "~~RULING (owner): \"yes the new trellis is approved.\"~~ withdrawn" => "remark",
        "`RULING` is a kind word" => "remark",
        "snake_case RULING" => "remark",
        "* note: RULING aside" => "remark",
        "**Lead**: the beds are dug" => "remark",
    ]
);

laws::table!(
    an_entry_names_its_quotes_cites_and_dates,
    |text: &&str| Line::read(1, text).read_by(Named),
    [
        "RULING (owner, explicit, 2026-11-05): \"the new trellis is approved.\" (approved)" => Ok([
            vec!["the new trellis is approved.".to_owned()],
            vec![],
            vec!["2026-11-05".to_owned()],
        ]),
        "- 2026-11-06 RULING (owner, 0a1b2c3d): \"every bed gets a label.\"" => Ok([
            vec!["every bed gets a label.".to_owned()],
            vec!["0a1b2c3d".to_owned()],
            vec!["2026-11-06".to_owned()],
        ]),
        "- 2026-11-06 RULING (owner, 0a1b2c3d): 'every bed gets a label.'" => Ok([
            vec![],
            vec!["0a1b2c3d".to_owned()],
            vec!["2026-11-06".to_owned()],
        ]),
        "Owner's note: keep it tidy, using Ann's tools, as \"don't guess\" says" => Ok([
            vec!["don't guess".to_owned()],
            vec![],
            vec![],
        ]),
        "Owner's note: keep it tidy, as 'don't guess' says" => Ok(<[Vec<String>; 3]>::default()),
        "- gardeners' reach and the workers' \"rain delay\" notes" => Ok([vec!["rain delay".to_owned()], vec![], vec![]]),
        "cited 1f2e3d4c-1234-4abc-8def-0123456789ab, deadbeef1 and 0123abcd:42 on 2026-11-07T21:09" => Ok([
            vec![],
            vec!["1f2e3d4c-1234-4abc-8def-0123456789ab".to_owned(), "0123abcd".to_owned()],
            vec!["2026-11-07".to_owned()],
        ]),
        "RULING (owner, 2026-11-05T04:07): \"dig the beds\" and 2026-11-06 08:15:30 too" => Ok([
            vec!["dig the beds".to_owned()],
            vec![],
            vec!["2026-11-05".to_owned(), "2026-11-06".to_owned()],
        ]),
        "an \"unclosed quote and a 'stray" => Err(Unclosed::new(Quote::new("unclosed quote and a 'stray".to_owned()))),
        "# Notes \"quoted\" 1f2e3d4c" => Ok([vec!["quoted".to_owned()], vec!["1f2e3d4c".to_owned()], vec![]]),
        "Lead: see ae0e9ace's note, c3c3c3c3's reply and Ann's" => Ok([
            vec![],
            vec!["ae0e9ace".to_owned(), "c3c3c3c3".to_owned()],
            vec![],
        ]),
    ]
);

laws::table!(
    a_single_quote_never_swallows_a_cite_or_date_after_its_close,
    |text: &&str| {
        let line = Line::read(1, text);
        (line.read_by(Shaped), line.read_by(Named))
    },
    [
        "RULING 'ship it' (0a1b2c3d), the owners' call" => ("ruling", Ok([vec![], vec!["0a1b2c3d".to_owned()], vec![]])),
        "RULING (owner): 'keep the owners' rules as they are' (0a1b2c3d, 2026-11-05)" => (
            "ruling",
            Ok([vec![], vec!["0a1b2c3d".to_owned()], vec!["2026-11-05".to_owned()]]),
        ),
        "RULING (owner): \"the owners' rules\" and 'beds' rows' on 2026-11-05" => (
            "ruling",
            Ok([vec!["the owners' rules".to_owned()], vec![], vec!["2026-11-05".to_owned()]]),
        ),
        "- 2026-11-06 lead (owner 0a1b2c3d: 'dig where you need'): the beds can't wait" => (
            "remark",
            Ok([vec![], vec!["0a1b2c3d".to_owned()], vec!["2026-11-06".to_owned()]]),
        ),
        "- RULING 2026-11-07 (0f1e2d3c): 'rock'n roll, and Ann's rake" => (
            "ruling",
            Ok([vec![], vec!["0f1e2d3c".to_owned()], vec!["2026-11-07".to_owned()]]),
        ),
        "a stray ' mark beside '0a1b2c3d' and the owners' sheds" => (
            "remark",
            Ok([vec![], vec!["0a1b2c3d".to_owned()], vec![]]),
        ),
        "RULING (owner, 'yes'): \"keep the owners' rules\" 'rock'n roll" => (
            "ruling",
            Ok([vec!["keep the owners' rules".to_owned()], vec![], vec![]]),
        ),
    ]
);

laws::table!(
    a_carriage_return_blank_line_is_layout,
    |text: &&str| {
        let notes = Notes::read(text);
        (notes.lines().iter().map(|line| line.read_by(Shaped)).collect::<Vec<&str>>(), notes.to_string() == *text)
    },
    [
        "# Notes\r\n\r\n  \r\n\t\r\nRULING (owner, 0a1b2c3d): kept\r\n" => (
            vec!["heading", "layout", "layout", "layout", "ruling"],
            true,
        ),
        "first\r\n \t \r\nlast" => (vec!["remark", "layout", "remark"], true),
        "\r\n" => (vec!["layout"], true),
        "x\ry\r\n" => (vec!["remark"], true),
    ]
);

laws::table!(
    a_notes_file_ends_as_its_last_content_line_ends,
    |text: &&str| Notes::read(text).ending(),
    [
        "# Notes\nlast line" => Ending::Open,
        "# Notes\nlast line\n" => Ending::Break,
        "# Notes\nlast line\r\n" => Ending::Break,
        "# Notes\nlast line\n  " => Ending::Break,
        "# Notes\nlast line\n\n" => Ending::Break,
        "  " => Ending::Break,
        "" => Ending::Break,
    ]
);

laws::table!(
    a_notes_file_without_a_final_newline_renders_back_to_its_exact_text,
    |lines: &&[&str]| {
        let text = lines.join("\n");
        Notes::read(&text).to_string() == text
    },
    [
        &["a", "b"][..] => true,
        &["# Notes", "last"][..] => true,
        &["RULING (owner): 'the owners' rules'", "  "][..] => true,
        &["RULING (owner): \"the owners' rules\"\r", "  \r", "\r"][..] => true,
        &[" \t"][..] => true,
        &[""][..] => true,
    ]
);
