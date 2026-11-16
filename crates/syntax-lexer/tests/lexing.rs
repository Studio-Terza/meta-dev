const OPENINGS: [&str; 5] = [
    "#![allow(dead_code)]\n//! Inner docs.\nfn f() {}\n",
    "\u{feff}fn f() {}\n",
    "#!/usr/bin/env run\nfn f() {\n    g();\n}\n",
    "\u{feff}#!/usr/bin/env run\n#![allow(dead_code)]\nstruct A;\n",
    "#! [allow(dead_code)]\nfn f() {}\n",
];

#[test]
fn lexing_once_parses_and_places_every_opening_as_parse_file_does() {
    let once: Vec<Option<syntax::File>> = OPENINGS
        .iter()
        .map(|content| syntax_lexer::Lexed::parsed(content).map(syntax_lexer::Lexed::into_file).ok())
        .collect();
    let direct: Vec<Option<syntax::File>> =
        OPENINGS.iter().map(|content| syntax::parse::parse_file(content).ok()).collect();
    let lines = |files: &[Option<syntax::File>]| -> Vec<Vec<usize>> {
        files
            .iter()
            .flatten()
            .map(|file| file.items.iter().map(|item| syntax::Spanned::span(item).start().line).collect())
            .collect()
    };
    assert_eq!(once, direct);
    assert_eq!(lines(&once), lines(&direct));
    assert_eq!(once.iter().flatten().count(), OPENINGS.len());
}

#[test]
fn every_literal_form_is_a_quoted_lexeme_and_words_and_marks_are_not() {
    let lexed = syntax_lexer::Lexed::parsed("const A: X = f(\"s\", 'c', b'x', b\"y\", r\"z\", 1.5, 7u8, -2);\n");
    let kinds: Vec<syntax_lexer::LexemeKind> = lexed.map_or_else(
        |_| Vec::new(),
        |mut found| found.tokens_taken().lexemes().iter().map(|lexeme| lexeme.kind()).collect(),
    );
    let quoted = kinds.iter().filter(|kind| kind.quoted().is_some()).count();
    assert_eq!([quoted, kinds.len()], [8, 25]);
}
