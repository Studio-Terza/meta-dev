use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Lexeme<'text> {
    #[regex(r"[ \t]+")]
    #[token("\\\n")]
    Blank,
    #[regex(r"#[^\n]*", allow_greedy = true)]
    Comment,
    #[token("\n")]
    Newline,
    #[token(";")]
    #[token("&")]
    #[token("&&")]
    #[token("||")]
    #[token("|")]
    #[token("|&")]
    Separator,
    #[token(";;")]
    #[token(";&")]
    #[token(";;&")]
    CaseEnd,
    #[token("(")]
    Open,
    #[regex(r"\([ \t]*\)")]
    Parens,
    #[token(")")]
    Close,
    #[token("((")]
    #[token("$((")]
    Arithmetic,
    #[token("))")]
    Closes,
    #[token("$(")]
    #[token("<(")]
    #[token(">(")]
    Substitution,
    #[token("`")]
    Backtick,
    #[regex(r"[0-9]*<<")]
    Heredoc,
    #[regex(r"[0-9]*<<-")]
    Stripped,
    #[regex(r"[0-9]*(<|<<<)")]
    Input,
    #[regex(r"[0-9]*(>|>>|>\||<>)")]
    #[token("&>")]
    #[token("&>>")]
    Output,
    #[regex(r"[0-9]*[<>]&([0-9]+-?|-)")]
    Duplicate,
    #[token("\"")]
    Quote,
    #[regex(r"'[^']*'", |lexer| Lexeme::inner(lexer.slice(), 1))]
    #[regex(r"\$'(\\(.|\n)|[^'\\])*'", |lexer| Lexeme::inner(lexer.slice(), 2))]
    Single(&'text str),
    #[regex(r"\\[^\n]", |lexer| lexer.slice().get(1..))]
    Escaped(&'text str),
    #[regex(r"\$\{[^}]*\}", |lexer| lexer.slice())]
    #[regex(r"\$([A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-])", |lexer| lexer.slice())]
    Expansion(&'text str),
    #[regex(r#"[^ \t\n;&|()<>'"`$\\#][^ \t\n;&|()<>'"`$\\]*"#, |lexer| lexer.slice())]
    #[token("$", |lexer| lexer.slice())]
    Literal(&'text str),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Quoted<'text> {
    #[token("\"")]
    End,
    #[token("$(")]
    Substitution,
    #[token("$((")]
    Arithmetic,
    #[token("`")]
    Backtick,
    #[regex(r"\\(.|\n)", |lexer| lexer.slice().get(1..))]
    Escaped(&'text str),
    #[regex(r"\$\{[^}]*\}", |lexer| lexer.slice())]
    #[regex(r"\$([A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-])", |lexer| lexer.slice())]
    #[regex(r#"[^"\\$`]+"#, |lexer| lexer.slice())]
    #[token("$", |lexer| lexer.slice())]
    Text(&'text str),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Line<'text> {
    #[regex(r"\t+")]
    Tabs,
    #[regex(r"[^\t\n][^\n]*", |lexer| lexer.slice(), allow_greedy = true)]
    Text(&'text str),
    #[token("\n")]
    End,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Spelling {
    #[token("for")]
    #[token("select")]
    For,
    #[token("while")]
    #[token("until")]
    While,
    #[token("if")]
    If,
    #[token("case")]
    Case,
    #[token("do")]
    #[token("then")]
    #[token("else")]
    #[token("elif")]
    #[token("!")]
    #[token("time")]
    Lead,
    #[token("{")]
    Brace,
    #[token("}")]
    Unbrace,
    #[token("done")]
    Done,
    #[token("fi")]
    Fi,
    #[token("esac")]
    Esac,
    #[token("function")]
    Function,
    #[token("[[")]
    Test,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*\+?=[^\x00]*", allow_greedy = true, priority = 100)]
    Assignment,
    #[regex(r"([^/\x00]*/)*rm", allow_greedy = true)]
    Rm,
    #[regex(r"([^/\x00]*/)*rmdir", allow_greedy = true)]
    Rmdir,
    #[regex(r"([^/\x00]*/)*(bash|sh|zsh|dash|ksh)", allow_greedy = true)]
    Shell,
    #[regex(r"([^/\x00]*/)*xargs", allow_greedy = true)]
    Xargs,
    #[regex(r"([^/\x00]*/)*find", allow_greedy = true)]
    Find,
    #[regex(r"([^/\x00]*/)*g?sed", allow_greedy = true)]
    Sed,
    #[regex(r"([^/\x00]*/)*perl", allow_greedy = true)]
    Perl,
    #[regex(r"([^/\x00]*/)*tee", allow_greedy = true)]
    Tee,
    #[regex(r"([^/\x00]*/)*mv", allow_greedy = true)]
    Mv,
    #[token("cd")]
    #[token("pushd")]
    Cd,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum SedFlag {
    #[regex(r"-[A-Za-z]*i[^\x00]*|--in-place[^\x00]*", allow_greedy = true)]
    InPlace,
    #[regex(r"-[^\x00]*", allow_greedy = true, priority = 1)]
    Other,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum PerlFlag {
    #[regex(r"-[a-hj-z]*i[^\x00]*", allow_greedy = true)]
    InPlace,
    #[regex(r"-[^\x00]*", allow_greedy = true, priority = 1)]
    Other,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
pub(super) enum Argued {
    #[token("-exec")]
    #[token("-execdir")]
    #[token("-ok")]
    #[token("-okdir")]
    Exec,
    #[token("-delete")]
    Delete,
    #[token(";")]
    #[token("+")]
    Terminator,
    #[token("in")]
    In,
    #[token("-I")]
    #[token("-J")]
    #[token("-L")]
    #[token("-n")]
    #[token("-P")]
    #[token("-s")]
    #[token("-E")]
    #[token("-d")]
    #[token("-a")]
    #[token("-R")]
    #[token("-S")]
    Valued,
    #[regex(r"-[A-Za-z]*c[A-Za-z]*", priority = 3)]
    Script,
    #[regex(r"[-+][^\x00]*", allow_greedy = true, priority = 1)]
    Flag,
}

#[derive(Debug)]
pub(super) struct Whole;

impl<'text> Lexeme<'text> {
    fn inner(slice: &'text str, opening: usize) -> Option<&'text str> {
        slice.get(opening..slice.len().saturating_sub(1))
    }
}

impl Whole {
    pub(super) fn read<'text, Token: Logos<'text, Source = str, Extras = ()> + Copy>(
        text: &'text str,
    ) -> Option<Token> {
        let mut lexer = Token::lexer(text);
        let first = lexer.next().and_then(|lexed| lexed.ignoring(Unlexed::NotThisGrammar));
        first.filter(|_token| lexer.span() == (0..text.len()))
    }
}
