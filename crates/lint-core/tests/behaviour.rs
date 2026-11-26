use std::path::Path;
use std::rc::Rc;

use lint_core::{Macros, Parsed, SourceFile, SourceText};
use syntax::{Expr, Item, Macro, Stmt};

const NESTED: &str = "fn f() {\n    outer!(inner!(a + b); c * d);\n}\n";

struct Invocations;

impl Invocations {
    fn in_item(item: &Item) -> Option<&Macro> {
        match item {
            Item::Fn(function) => Self::first(&function.block.stmts),
            _ => None,
        }
    }

    fn first(statements: &[Stmt]) -> Option<&Macro> {
        statements.iter().find_map(Self::in_statement)
    }

    fn in_statement(statement: &Stmt) -> Option<&Macro> {
        match statement {
            Stmt::Macro(found) => Some(&found.mac),
            Stmt::Expr(Expr::Macro(found), _) => Some(&found.mac),
            _ => None,
        }
    }
}

struct Walked;

impl Walked {
    fn statements(file: &SourceFile, invocation: &Macro) -> Vec<String> {
        let mut spoken = Vec::new();
        Macros::walk(file, invocation, |statement| spoken.push(format!("{statement:?}")));
        spoken
    }
}

#[test]
fn a_macro_body_walks_to_the_same_statements_each_time_from_any_copy_and_when_nested() -> Result<(), laws::Unmet> {
    let file = SourceFile::parse("crates/x/src/lib.rs", SourceText::new(NESTED.to_owned()))?;
    let outer = file.syntax().items.iter().find_map(Invocations::in_item).ok_or("no macro in the file")?;
    Macros::pin(&file, outer);
    let copied = outer.clone();
    let first = Walked::statements(&file, outer);
    let expansion = Macros::expanded(&file, outer);
    let inner = Invocations::first(&expansion).ok_or("no macro inside the expansion")?;
    Macros::pin(&file, inner);
    let inner_copy = inner.clone();
    assert_eq!(first.len(), 2);
    assert_eq!([Walked::statements(&file, outer), Walked::statements(&file, &copied)], [first.clone(), first]);
    assert_eq!(Walked::statements(&file, inner).len(), 1);
    assert_eq!(Walked::statements(&file, inner), Walked::statements(&file, &inner_copy));
    assert!(Rc::ptr_eq(&Macros::expanded(&file, inner), &Macros::expanded(&file, inner)));
    Ok(())
}

#[test]
fn a_macro_outside_the_file_is_expanded_afresh_each_time() -> Result<(), laws::Unmet> {
    let file = SourceFile::parse("crates/x/src/lib.rs", SourceText::new(NESTED.to_owned()))?;
    let lengths = ["m!(a)", "m!(a; b)", "m!(a; b; c)"]
        .iter()
        .map(|text| {
            syntax::parse::parse_str::<Macro>(text).map(|invocation| Macros::expanded(&file, &invocation).len())
        })
        .collect::<Result<Vec<usize>, syntax::Error>>()?;
    assert_eq!(lengths, [1, 2, 3]);
    Ok(())
}

#[test]
fn a_parsed_file_pairs_each_test_module_with_its_own_head() -> Result<(), laws::Unmet> {
    let module = SourceFile::parse("crates/x/src/tests.rs", SourceText::new("fn checked() {}\n".to_owned()))?;
    let file = SourceFile::parse("crates/x/src/lib.rs", SourceText::new("mod tests;\n".to_owned()))?
        .with_test_modules(vec![module]);
    let paths: Vec<String> = Parsed::from(&file)
        .test_modules()
        .into_iter()
        .map(|parsed| AsRef::<Path>::as_ref(parsed.head()).display().to_string())
        .collect();
    assert_eq!(paths, ["crates/x/src/tests.rs"]);
    Ok(())
}

#[test]
fn a_pinned_macro_reads_its_nested_calls_once_and_a_copy_reads_them_afresh() -> Result<(), laws::Unmet> {
    let file = SourceFile::parse("crates/x/src/lib.rs", SourceText::new(NESTED.to_owned()))?;
    let outer = file.syntax().items.iter().find_map(Invocations::in_item).ok_or("no macro in the file")?;
    let copied = outer.clone();
    Macros::pin(&file, outer);
    let nested = Macros::nested(&file, outer);
    let inner = nested.first().ok_or("no macro nested in the call")?;
    assert!(Rc::ptr_eq(&nested, &Macros::nested(&file, outer)));
    assert!(Rc::ptr_eq(&Macros::nested(&file, inner), &Macros::nested(&file, inner)));
    assert_ne!(Rc::as_ptr(&Macros::nested(&file, &copied)), Rc::as_ptr(&Macros::nested(&file, &copied)));
    assert_eq!(Macros::nested(&file, &copied).len(), nested.len());
    Ok(())
}

const RULES: &str = "macro_rules! made {\n    ($($name:ident),*) => {\n        $(impl $crate::Kind for $name {\n            fn f() -> ::a::B { c::d() }\n        })*\n        fn tail() -> e::F {}\n    };\n}";

const NOT_RULES: &str = "other! {\n    fn f() -> a::B {}\n}";

#[test]
fn a_macro_rules_transcriber_mentions_its_paths_with_each_metavariable_as_its_name() -> Result<(), laws::Unmet> {
    let rules: syntax::ItemMacro = syntax::parse::parse_str(RULES)?;
    let other: syntax::ItemMacro = syntax::parse::parse_str(NOT_RULES)?;
    let mentioned: Vec<String> = Macros::mentioned(&rules.mac)
        .paths()
        .iter()
        .map(|path| syntax::ToTokens::to_token_stream(path).to_string())
        .collect();
    assert_eq!(mentioned, ["crate :: Kind", "name", ":: a :: B", "c :: d", "e :: F"]);
    assert_eq!(Macros::mentioned(&other.mac).paths().iter().count(), 0);
    Ok(())
}
