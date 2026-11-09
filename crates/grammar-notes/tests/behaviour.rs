#![deny(clippy::wildcard_enum_match_arm)]

use grammar_notes::{Date, Ending, Entry, Line, Notes, Quote, Reading, Unclosed};
use laws::proptest::collection::vec;
use laws::proptest::prelude::{Strategy, any, prop_assert_eq, prop_oneof};
use laws::proptest::sample::select;

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
        let (drawn, (ending, ended)) = file;
        let lines: Vec<(String, String)> = drawn.into_iter().chain([("x".to_owned(), "x".to_owned())]).collect();
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
        quotes.map(|closed| {
            [
                closed,
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

laws::table!(
    a_date_is_exactly_four_digits_then_two_then_two_joined_by_dashes,
    |day: &&str| Date::try_new((*day).to_owned()).ok().map(Date::into_inner),
    [
        "2026-11-05" => Some("2026-11-05".to_owned()),
        "" => None,
        "26-11-05" => None,
        "2026-1-05" => None,
        "2026-11-5" => None,
        "2026/11/05" => None,
        "2026-11-05x" => None,
        "202a-11-05" => None,
        "2026-11-05 2026-11-06" => None,
    ]
);

laws::table!(
    a_date_parses_from_text_only_when_it_is_spelled_as_one,
    |day: &&str| day.parse::<Date>().ok().map(|date| date.to_string()),
    [
        "1999-12-31" => Some("1999-12-31".to_owned()),
        "1999-12" => None,
        "1999-12-311" => None,
        "19991231" => None,
    ]
);

laws::roundtrip!(
    a_date_is_displayed_and_parsed_back,
    any::<Date>(),
    |date: &Date| date.to_string(),
    |text: &String| text.parse::<Date>().ok().map(|date| date.to_string()),
    |date: Date| Some(date.to_string())
);

laws::table!(
    a_date_is_any_year_month_day_wherever_it_stands_and_whatever_follows_it,
    |text: &&str| Line::read(1, text).read_by(Named),
    [
        "ruled 2026-11-05\"rain delay\" (0a1b2c3d)" => Ok([
            vec!["rain delay".to_owned()],
            vec!["0a1b2c3d".to_owned()],
            vec!["2026-11-05".to_owned()],
        ]),
        "ruled 2026-11-05T04:07\"rain delay\" (0a1b2c3d)" => Ok([
            vec!["rain delay".to_owned()],
            vec!["0a1b2c3d".to_owned()],
            vec!["2026-11-05".to_owned()],
        ]),
        "ruled 2026-11-05 04:07:09.5+01:00 \"rain delay\"" => Ok([
            vec!["rain delay".to_owned()],
            vec![],
            vec!["2026-11-05".to_owned()],
        ]),
        "ruled 2026-11-05(0a1b2c3d)" => Ok([vec![], vec!["0a1b2c3d".to_owned()], vec!["2026-11-05".to_owned()]]),
        "ruled 2026-11-05: \"rain delay\"" => Ok([
            vec!["rain delay".to_owned()],
            vec![],
            vec!["2026-11-05".to_owned()],
        ]),
        "ruled 2026-11-05 and 2026-11-06 and 2026-12-31." => Ok([
            vec![],
            vec![],
            vec!["2026-11-05".to_owned(), "2026-11-06".to_owned(), "2026-12-31".to_owned()],
        ]),
        "ruled 2026-1-05, 26-11-05, 2026-11-5, 2026/11/05 and 202611050" => Ok(<[Vec<String>; 3]>::default()),
        "ruled at T04:07 or 04:07 or 04:07:09Z with \"rain delay\"" => Ok([
            vec!["rain delay".to_owned()],
            vec![],
            vec![],
        ]),
    ]
);
