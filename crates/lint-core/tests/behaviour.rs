use std::error::Error;
use std::rc::Rc;

use lint_core::{Macros, Parsed, SourceFile, SourceText};
use syn::{Expr, Item, Macro, Stmt};

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
fn a_macro_body_walks_to_the_same_statements_each_time_from_any_copy_and_when_nested() -> Result<(), Box<dyn Error>> {
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
fn a_macro_outside_the_file_is_expanded_afresh_each_time() -> Result<(), Box<dyn Error>> {
    let file = SourceFile::parse("crates/x/src/lib.rs", SourceText::new(NESTED.to_owned()))?;
    let lengths = ["m!(a)", "m!(a; b)", "m!(a; b; c)"]
        .iter()
        .map(|text| syn::parse_str::<Macro>(text).map(|invocation| Macros::expanded(&file, &invocation).len()))
        .collect::<Result<Vec<usize>, syn::Error>>()?;
    assert_eq!(lengths, [1, 2, 3]);
    Ok(())
}

#[test]
fn a_parsed_file_pairs_each_test_module_with_its_own_head() -> Result<(), Box<dyn Error>> {
    let module = SourceFile::parse("crates/x/src/tests.rs", SourceText::new("fn checked() {}\n".to_owned()))?;
    let file = SourceFile::parse("crates/x/src/lib.rs", SourceText::new("mod tests;\n".to_owned()))?
        .with_test_modules(vec![module]);
    let paths: Vec<String> = Parsed::from(&file)
        .test_modules()
        .into_iter()
        .map(|parsed| parsed.head().path().display().to_string())
        .collect();
    assert_eq!(paths, ["crates/x/src/tests.rs"]);
    Ok(())
}
