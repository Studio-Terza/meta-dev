use syn::{Block, Expr, Pat, Stmt, Type};

#[derive(Clone, Copy, Debug)]
pub struct Past<'tree, Node>(&'tree Node);

impl<'tree, Node> Past<'tree, Node> {
    #[must_use]
    pub fn node(self) -> &'tree Node {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct Spine<'tree>(Option<&'tree Expr>);

#[derive(Clone, Debug)]
pub struct TypeSpine<'tree>(Option<&'tree Type>);

#[derive(Clone, Debug)]
pub struct PatSpine<'tree>(Option<&'tree Pat>);

impl<'tree> Spine<'tree> {
    pub fn past(mut self, through: impl Fn(&Expr) -> bool) -> Option<Past<'tree, Expr>> {
        self.find(|node| through(node).then_some(()).is_none()).map(Past)
    }

    fn inner(expr: &'tree Expr) -> Option<&'tree Expr> {
        match expr {
            Expr::Paren(inner) => Some(&inner.expr),
            Expr::Group(inner) => Some(&inner.expr),
            Expr::Reference(inner) => Some(&inner.expr),
            Expr::Unary(inner) => Some(&inner.expr),
            Expr::Field(inner) => Some(&inner.base),
            Expr::Index(inner) => Some(&inner.expr),
            Expr::MethodCall(inner) => Some(&inner.receiver),
            Expr::Try(inner) => Some(&inner.expr),
            Expr::Cast(inner) => Some(&inner.expr),
            Expr::Await(inner) => Some(&inner.base),
            Expr::Block(block) => Self::tail(&block.block),
            _ => None,
        }
    }
}

impl<'tree> Spine<'tree> {
    fn tail(block: &'tree Block) -> Option<&'tree Expr> {
        let only = block.stmts.first().filter(|_| block.stmts.len() == 1)?;
        Self::tail_expr(only)
    }

    fn tail_expr(statement: &'tree Stmt) -> Option<&'tree Expr> {
        match statement {
            Stmt::Expr(tail, None) => Some(tail),
            _ => None,
        }
    }
}

impl<'tree> From<&'tree Expr> for Spine<'tree> {
    fn from(expr: &'tree Expr) -> Spine<'tree> {
        Spine(Some(expr))
    }
}

impl<'tree> From<&'tree Type> for TypeSpine<'tree> {
    fn from(ty: &'tree Type) -> TypeSpine<'tree> {
        TypeSpine(Some(ty))
    }
}

impl<'tree> From<&'tree Pat> for PatSpine<'tree> {
    fn from(pat: &'tree Pat) -> PatSpine<'tree> {
        PatSpine(Some(pat))
    }
}

impl<'tree> Iterator for Spine<'tree> {
    type Item = &'tree Expr;

    fn next(&mut self) -> Option<&'tree Expr> {
        let current = self.0?;
        self.0 = Self::inner(current);
        Some(current)
    }
}

impl<'tree> TypeSpine<'tree> {
    pub fn past(mut self, through: impl Fn(&Type) -> bool) -> Option<Past<'tree, Type>> {
        self.find(|node| through(node).then_some(()).is_none()).map(Past)
    }

    fn inner(ty: &'tree Type) -> Option<&'tree Type> {
        match ty {
            Type::Paren(inner) => Some(&inner.elem),
            Type::Group(inner) => Some(&inner.elem),
            Type::Reference(inner) => Some(&inner.elem),
            Type::Ptr(inner) => Some(&inner.elem),
            Type::Slice(inner) => Some(&inner.elem),
            Type::Array(inner) => Some(&inner.elem),
            _ => None,
        }
    }
}

impl<'tree> Iterator for TypeSpine<'tree> {
    type Item = &'tree Type;

    fn next(&mut self) -> Option<&'tree Type> {
        let current = self.0?;
        self.0 = Self::inner(current);
        Some(current)
    }
}

impl<'tree> PatSpine<'tree> {
    pub fn past(mut self, through: impl Fn(&Pat) -> bool) -> Option<Past<'tree, Pat>> {
        self.find(|node| through(node).then_some(()).is_none()).map(Past)
    }

    fn inner(pat: &'tree Pat) -> Option<&'tree Pat> {
        match pat {
            Pat::Paren(inner) => Some(&inner.pat),
            Pat::Reference(inner) => Some(&inner.pat),
            Pat::Type(inner) => Some(&inner.pat),
            Pat::Ident(binding) => binding.subpat.as_ref().map(|(_, inner)| &**inner),
            _ => None,
        }
    }
}

impl<'tree> Iterator for PatSpine<'tree> {
    type Item = &'tree Pat;

    fn next(&mut self) -> Option<&'tree Pat> {
        let current = self.0?;
        self.0 = Self::inner(current);
        Some(current)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatEdge {
    Root,
    Alternative,
    Inner,
    Field,
    Element,
}

#[derive(Clone, Copy, Debug)]
pub struct PatReached<'pat> {
    pat: &'pat Pat,
    edge: PatEdge,
}

#[derive(Debug)]
pub struct PatTree;

impl<'pat> PatReached<'pat> {
    fn along(pat: &'pat Pat, edge: PatEdge) -> PatReached<'pat> {
        PatReached { pat, edge }
    }

    #[must_use]
    pub fn pat(self) -> &'pat Pat {
        self.pat
    }

    #[must_use]
    pub fn edge(self) -> PatEdge {
        self.edge
    }
}

impl PatTree {
    pub fn reached<'pat>(pat: &'pat Pat, opens: &impl Fn(&Pat, PatEdge) -> bool) -> Vec<PatReached<'pat>> {
        Self::from_edge(pat, PatEdge::Root, opens)
    }

    fn from_edge<'pat>(pat: &'pat Pat, edge: PatEdge, opens: &impl Fn(&Pat, PatEdge) -> bool) -> Vec<PatReached<'pat>> {
        let here = PatReached { pat, edge };
        let children = Some(()).filter(|()| opens(pat, edge)).map_or_default(|()| Self::children(pat));
        let below = children.into_iter().flat_map(|child| Self::from_edge(child.pat, child.edge, opens));
        [here].into_iter().chain(below).collect()
    }

    fn children(pat: &Pat) -> Vec<PatReached<'_>> {
        match pat {
            Pat::Or(alternatives) => {
                alternatives.cases.iter().map(|case| PatReached::along(case, PatEdge::Alternative)).collect()
            }
            Pat::Reference(inner) => vec![PatReached::along(&inner.pat, PatEdge::Inner)],
            Pat::Paren(inner) => vec![PatReached::along(&inner.pat, PatEdge::Inner)],
            Pat::Type(inner) => vec![PatReached::along(&inner.pat, PatEdge::Inner)],
            Pat::Ident(binding) => {
                binding.subpat.iter().map(|(_, inner)| PatReached::along(inner.as_ref(), PatEdge::Inner)).collect()
            }
            Pat::TupleStruct(variant) => {
                variant.elems.iter().map(|elem| PatReached::along(elem, PatEdge::Field)).collect()
            }
            Pat::Struct(variant) => {
                variant.fields.iter().map(|field| PatReached::along(&field.pat, PatEdge::Field)).collect()
            }
            Pat::Tuple(tuple) => tuple.elems.iter().map(|elem| PatReached::along(elem, PatEdge::Element)).collect(),
            Pat::Slice(slice) => slice.elems.iter().map(|elem| PatReached::along(elem, PatEdge::Element)).collect(),
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprShape {
    text: String,
    leaves: Vec<syn::Ident>,
}

#[derive(Default)]
struct Leaves(Vec<syn::Ident>);

impl<'expr> syn::visit::Visit<'expr> for Leaves {
    fn visit_expr_path(&mut self, path: &'expr syn::ExprPath) {
        self.0.extend(path.path.get_ident().cloned());
    }
}

impl From<&Expr> for ExprShape {
    fn from(expr: &Expr) -> ExprShape {
        let mut leaves = Leaves::default();
        syn::visit::Visit::visit_expr(&mut leaves, expr);
        let text = ExprShape::rendered(quote::ToTokens::to_token_stream(expr), &leaves.0);
        ExprShape { text, leaves: leaves.0 }
    }
}

impl ExprShape {
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn leaves(&self) -> &[syn::Ident] {
        &self.leaves
    }

    fn rendered(tokens: proc_macro2::TokenStream, leaves: &[syn::Ident]) -> String {
        tokens.into_iter().map(|token| Self::token(token, leaves)).collect::<Vec<String>>().join(" ")
    }

    fn token(token: proc_macro2::TokenTree, leaves: &[syn::Ident]) -> String {
        match token {
            proc_macro2::TokenTree::Ident(ident) => {
                leaves.iter().find(|leaf| **leaf == ident).map_or_else(|| ident.to_string(), |_| "_".to_owned())
            }
            proc_macro2::TokenTree::Group(group) => Self::group(&group, leaves),
            proc_macro2::TokenTree::Punct(punct) => punct.as_char().to_string(),
            proc_macro2::TokenTree::Literal(literal) => literal.to_string(),
        }
    }

    fn group(group: &proc_macro2::Group, leaves: &[syn::Ident]) -> String {
        let [open, close] = Self::delimiters(group.delimiter());
        [open, Self::rendered(group.stream(), leaves).as_str(), close].concat()
    }

    fn delimiters(delimiter: proc_macro2::Delimiter) -> [&'static str; 2] {
        match delimiter {
            proc_macro2::Delimiter::Parenthesis => ["(", ")"],
            proc_macro2::Delimiter::Brace => ["{", "}"],
            proc_macro2::Delimiter::Bracket => ["[", "]"],
            proc_macro2::Delimiter::None => ["", ""],
        }
    }
}
