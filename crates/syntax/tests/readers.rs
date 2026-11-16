use syntax::parse::{Parser, parse_str};
use syntax::{
    Application, Applied, Constant, Constructed, Error, Expr, ExprMacro, File, Ident, Macro, MacroTree, Meta, Nested,
    Pat, PatEdge, PatSpine, PatTree, Spine, ToTokens, Type, TypeFold, TypeSpine, TypeTree, UseLeaf, UseTree,
};

#[test]
fn use_leaves_carry_their_path_and_the_name_they_bind() -> Result<(), Error> {
    let tree: UseTree = parse_str("a::{b, c as d, e::*}")?;
    let leaves: Vec<String> = UseLeaf::of_tree(&tree)
        .iter()
        .map(|leaf| {
            let path: Vec<String> = leaf.path().iter().map(ToString::to_string).collect();
            format!("{} {:?}", path.join("::"), leaf.end().bound().map(ToString::to_string))
        })
        .collect();
    assert_eq!(leaves, ["a Some(\"b\")", "a Some(\"d\")", "a::e None"]);
    Ok(())
}

#[test]
fn spines_stop_at_the_first_node_not_passed_through() -> Result<(), Error> {
    let expr: Expr = parse_str("(&a.b).c()?")?;
    let spine: Vec<String> = Spine::from(&expr).map(|node| node.to_token_stream().to_string()).collect();
    assert_eq!(spine, ["(& a . b) . c () ?", "(& a . b) . c ()", "(& a . b)", "& a . b", "a . b", "a"]);
    let peeled = Spine::from(&expr).past(|node| matches!(node, Expr::Try(_) | Expr::MethodCall(_) | Expr::Paren(_)));
    assert_eq!(peeled.map(|found| found.node().to_token_stream().to_string()), Some("& a . b".to_owned()));
    let ty: Type = parse_str("&[Vec<u8>]")?;
    let reached = TypeSpine::from(&ty).past(|node| matches!(node, Type::Reference(_) | Type::Slice(_)));
    assert_eq!(reached.map(|found| found.node().to_token_stream().to_string()), Some("Vec < u8 >".to_owned()));
    let pat: Pat = Pat::parse_single.parse_str("&(x @ Some(_))")?;
    let bound = PatSpine::from(&pat).past(|node| matches!(node, Pat::Reference(_) | Pat::Paren(_) | Pat::Ident(_)));
    assert_eq!(bound.map(|found| found.node().to_token_stream().to_string()), Some("Some (_)".to_owned()));
    Ok(())
}

#[test]
fn pattern_trees_reach_what_the_caller_opens_in_preorder() -> Result<(), Error> {
    let pat: Pat = Pat::parse_multi.parse_str("Some((a, b)) | c")?;
    let opened = |found: &Pat, _: PatEdge| matches!(found, Pat::Or(_) | Pat::TupleStruct(_));
    let reached: Vec<String> = PatTree::reached(&pat, &opened)
        .into_iter()
        .map(|found| format!("{:?} {}", found.edge(), found.pat().to_token_stream()))
        .collect();
    assert_eq!(reached, ["Root Some ((a , b)) | c", "Alternative Some ((a , b))", "Field (a , b)", "Alternative c"]);
    Ok(())
}

#[test]
fn applied_metas_unwrap_cfg_attr() -> Result<(), Error> {
    let meta: Meta = parse_str("cfg_attr(test, allow(dead_code), deny(unused))")?;
    let metas: Vec<String> = Applied::metas(&meta).iter().map(|found| found.to_token_stream().to_string()).collect();
    assert_eq!(metas, ["allow (dead_code)", "deny (unused)"]);
    let plain: Meta = parse_str("allow(dead_code)")?;
    assert_eq!(Applied::metas(&plain), [plain]);
    Ok(())
}

#[test]
fn nested_modules_know_their_ancestors() -> Result<(), Error> {
    let file: File = parse_str("mod a { mod b { mod c; } } mod d;")?;
    let modules: Vec<String> = Nested::modules(&file.items)
        .iter()
        .map(|nested| {
            let ancestors: Vec<String> = nested.ancestors().iter().map(|outer| outer.ident.to_string()).collect();
            format!("{} in {}", nested.module().ident, ancestors.join("::"))
        })
        .collect();
    assert_eq!(modules, ["a in ", "b in a", "c in a::b", "d in "]);
    Ok(())
}

#[test]
fn macro_trees_expand_items_and_walk_nested_invocations() -> Result<(), Error> {
    let file: File = parse_str("outer! { fn f() {} inner! { fn g() {} } } named! m { fn h() {} }")?;
    let expanded: Vec<String> =
        MacroTree::expanded_items(&file.items).iter().map(|item| item.to_token_stream().to_string()).collect();
    assert_eq!(expanded, ["fn f () { }", "inner ! { fn g () { } }", "fn g () { }"]);
    let root: ExprMacro = parse_str("a!(b!(c!(d)))")?;
    let nested =
        |invocation: &Macro| invocation.parse_body::<ExprMacro>().map_or_else(|_| Vec::new(), |inner| vec![inner.mac]);
    let named =
        |found: &[Macro]| found.iter().map(|invocation| invocation.path.to_token_stream().to_string()).collect();
    let preorder: Vec<String> = named(&MacroTree::preorder(&root.mac, &nested));
    assert_eq!(preorder, ["a", "b", "c"]);
    let stops = |invocation: &Macro| invocation.path.is_ident("c").then_some(());
    let frontier: Vec<String> = named(&MacroTree::frontier(&root.mac, &nested, &stops));
    assert_eq!(frontier, ["c"]);
    Ok(())
}

#[test]
fn type_folds_see_children_before_parents_and_hand_each_out_once() -> Result<(), Error> {
    let ty: Type = parse_str("&(Vec<u8>, [i32; 2])")?;
    let fold = TypeFold::over(TypeTree::from(&ty), Rendered::step);
    assert_eq!(fold.take(&ty), Some("ref (Vec < u8 >, array i32)".to_owned()));
    assert_eq!(fold.take(&ty), None);
    Ok(())
}

#[test]
fn constants_are_literals_and_paths_in_tuples_and_arrays() -> Result<(), Error> {
    let table: Expr = parse_str("[(1, -A::B), (2, (C))]")?;
    assert!(Constant::composed(&table).is_some());
    let computed: Expr = parse_str("[(1, f())]")?;
    assert!(Constant::composed(&computed).is_none());
    Ok(())
}

#[test]
fn applications_lead_back_to_one_root() -> Result<(), Error> {
    let chained: Expr = parse_str("wrap(&(a.b).len(c), d)")?;
    let root = Application::root(&chained).map(|found| found.to_token_stream().to_string());
    assert_eq!(root, Some("a".to_owned()));
    let literal: Expr = parse_str("a.get(1)")?;
    assert!(Application::root(&literal).is_none());
    let empty: Expr = parse_str("make()")?;
    assert!(Application::root(&empty).is_none());
    Ok(())
}

#[test]
fn constructed_values_open_only_the_named_wrappers_and_conversions() -> Result<(), Error> {
    let built: Expr = parse_str("Some([&x.to_owned(), (y, S { f: z })]).other(w)")?;
    let wraps = |function: &Expr| matches!(function, Expr::Path(path) if path.path.is_ident("Some")).then_some(());
    let converts = |method: &Ident| (*method == "to_owned").then_some(());
    let parts = Constructed::new(&wraps, &converts);
    let shown = |found: &Expr| -> Vec<String> {
        parts.parts(found).iter().map(|part| part.to_token_stream().to_string()).collect()
    };
    assert_eq!(shown(&built), ["Some ([& x . to_owned () , (y , S { f : z })]) . other (w)"]);
    let receiver: Expr = parse_str("[&x.to_owned(), (y, S { f: z })]")?;
    assert_eq!(shown(&receiver), ["x", "y", "z"]);
    Ok(())
}

struct Rendered;

impl Rendered {
    fn step(done: &TypeFold<'_, String>, node: &Type) -> String {
        match node {
            Type::Reference(reference) => format!("ref {}", done.take(&reference.elem).unwrap_or_default()),
            Type::Tuple(tuple) => Rendered::tuple(done, tuple.elems.iter()),
            Type::Array(array) => format!("array {}", done.take(&array.elem).unwrap_or_default()),
            _ => node.to_token_stream().to_string(),
        }
    }

    fn tuple<'ty>(done: &TypeFold<'_, String>, parts: impl Iterator<Item = &'ty Type>) -> String {
        let shown: Vec<String> = parts.map(|part| done.take(part).unwrap_or_default()).collect();
        format!("({})", shown.join(", "))
    }
}
