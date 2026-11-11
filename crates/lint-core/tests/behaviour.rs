use std::error::Error;
use std::rc::Rc;

use lint_core::{Macros, SourceFile, SourceText};
use syn::Macro;
use syn::visit::Visit;

const NESTED: &str = "fn f() {\n    outer!(inner!(a + b); c * d);\n}\n";

#[derive(Default)]
struct Invocations<'ast>(Vec<&'ast Macro>);

impl<'ast> Visit<'ast> for Invocations<'ast> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.0.push(invocation);
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
    let mut found = Invocations::default();
    found.visit_file(file.syntax());
    let outer = found.0.first().copied().ok_or("no macro in the file")?;
    let copied = outer.clone();
    let first = Walked::statements(&file, outer);
    let expansion = Macros::expanded(&file, outer);
    let nested = expansion.iter().fold(Invocations::default(), |mut inside, statement| {
        inside.visit_stmt(statement);
        inside
    });
    let inner = nested.0.first().copied().ok_or("no macro inside the expansion")?;
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
