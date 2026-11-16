mod literal;

use std::borrow::Cow;
use std::collections::BTreeSet;

use syntax::Punctuated;
use syntax::{Expr, ExprEdge, ExprTree, Macro, Token};

use lint_core::Syntax;
use lint_layout::{Name, Whole, Word, words};

use super::bindings::Bindings;
use super::pinning::Pinning;
use literal::{Literal, Reading};

#[derive(Debug)]
enum Step<'side> {
    Found(Option<Pinning>),
    Bound(Name),
    Tree(Box<Cow<'side, Expr>>),
}

impl Step<'_> {
    const PEELED: [Word<'static>; 8] =
        words!["to_owned", "to_string", "into", "as_str", "clone", "as_bytes", "to_vec", "as_ref"];
    const WRAPPERS: [Word<'static>; 5] = words!["Some", "Ok", "Err", "from", "new"];
    const FORMATS: [Word<'static>; 2] = words!["format", "format_args"];
    const LISTS: [Word<'static>; 2] = words!["concat", "vec"];

    fn opens(parent: &Expr, edge: ExprEdge) -> Option<()> {
        match parent {
            Expr::Reference(_) | Expr::Paren(_) | Expr::Array(_) | Expr::Tuple(_) | Expr::Struct(_) => Some(()),
            Expr::MethodCall(call) => Name::from(&call.method)
                .matches::<Whole>(&Self::PEELED)
                .filter(|_| edge == ExprEdge::Receiver)
                .map(|_| ()),
            Expr::Call(call) => Literal::called(&call.func)
                .and_then(|last| last.matches::<Whole>(&Self::WRAPPERS))
                .filter(|_| edge != ExprEdge::Function)
                .map(|_| ()),
            _ => None,
        }
    }

    fn leaf<'any>(node: &Expr) -> Vec<Step<'any>> {
        match node {
            Expr::Lit(literal) => vec![Step::Found(Literal::pinning(&literal.lit, Reading::Plain))],
            Expr::Macro(invocation) => Step::built(&invocation.mac),
            Expr::Path(path) => path.path.get_ident().map(Name::from).map(Step::Bound).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn built<'any>(invocation: &Macro) -> Vec<Step<'any>> {
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident));
        let arguments: Vec<Expr> = invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map_or_default(|arguments| arguments.into_iter().collect());
        let formatted = name.as_ref().and_then(|named| named.matches::<Whole>(&Self::FORMATS)).map(|_| {
            let literal = arguments.first().and_then(Syntax::literal);
            vec![Step::Found(literal.and_then(|present| Literal::pinning(present, Reading::Format)))]
        });
        let listed = name
            .as_ref()
            .and_then(|named| named.matches::<Whole>(&Self::LISTS))
            .map(|_| arguments.into_iter().map(|argument| Step::Tree(Box::new(Cow::Owned(argument)))).collect());
        formatted.or(listed).unwrap_or_default()
    }
}

#[derive(Debug)]
struct Opened<'side> {
    found: Option<Pinning>,
    steps: Vec<Step<'side>>,
}

impl<'side> Opened<'side> {
    fn found(found: Option<Pinning>) -> Opened<'side> {
        Opened { found, steps: Vec::new() }
    }

    fn tree(tree: &Expr) -> Opened<'side> {
        let reached = ExprTree::reached(tree, &|parent, edge| Step::opens(parent, edge).is_some());
        Opened { found: None, steps: reached.iter().flat_map(|node| Step::leaf(node.expr())).collect() }
    }
}

#[derive(Debug)]
struct Trace<'run, 'side> {
    compared: &'side Compared<'run>,
    seen: BTreeSet<Name>,
    pending: Vec<Step<'side>>,
}

impl<'run: 'side, 'side> Trace<'run, 'side> {
    fn opened(&mut self, step: Step<'side>) -> Opened<'side> {
        match step {
            Step::Found(found) => Opened::found(found),
            Step::Bound(name) => self.bound(name),
            Step::Tree(tree) => Opened::tree(&tree),
        }
    }

    fn bound(&mut self, name: Name) -> Opened<'side> {
        let value = self.compared.locals.bound(&name).or_else(|| self.compared.constants.bound(&name));
        let fresh = value.filter(|_| self.seen.insert(name));
        Opened { found: None, steps: fresh.map(|tree| Step::Tree(Box::new(Cow::Borrowed(tree)))).into_iter().collect() }
    }
}

impl<'run: 'side, 'side> Iterator for Trace<'run, 'side> {
    type Item = Option<Pinning>;

    fn next(&mut self) -> Option<Option<Pinning>> {
        let step = self.pending.pop()?;
        let opened = self.opened(step);
        self.pending.extend(opened.steps.into_iter().rev());
        Some(opened.found)
    }
}

#[derive(Debug)]
pub struct Compared<'run> {
    constants: &'run Bindings<'run>,
    locals: &'run Bindings<'run>,
}

impl<'run> Compared<'run> {
    pub fn new(constants: &'run Bindings<'run>, locals: &'run Bindings<'run>) -> Compared<'run> {
        Compared { constants, locals }
    }

    pub fn pinning(&self, side: &Expr) -> Option<Pinning> {
        let pending = vec![Step::Tree(Box::new(Cow::Borrowed(side)))];
        Trace { compared: self, seen: BTreeSet::new(), pending }.flatten().next()
    }
}
