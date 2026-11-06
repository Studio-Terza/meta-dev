mod written;

use proc_macro2::Span;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{Error, Ident, PathSegment, ReturnType, Type};

use written::Written;

pub enum Returned {
    Plain(ReturnType),
    Acquired(ReturnType),
}

enum Shape {
    Plain,
    Acquired,
    Refused,
}

struct HeldNames(Vec<Ident>);

struct Released;

impl<'tree> Visit<'tree> for HeldNames {
    fn visit_path_segment(&mut self, segment: &'tree PathSegment) {
        self.0.extend(Some(&segment.ident).filter(|name| *name == "Held").cloned());
        syn::visit::visit_path_segment(self, segment);
    }
}

impl VisitMut for Released {
    fn visit_type_mut(&mut self, tree: &mut Type) {
        syn::visit_mut::visit_type_mut(self, tree);
        *tree = Written::reading(tree).resource().cloned().unwrap_or_else(|| tree.clone());
    }
}

impl Shape {
    fn judged(output: &ReturnType) -> Shape {
        let mut names = HeldNames(Vec::new());
        names.visit_return_type(output);
        let found = names.0.len();
        let ending = Returned::tree(output).and_then(|tree| Written::reading(tree).spine_end().resource());
        (found == 0)
            .then_some(Shape::Plain)
            .or_else(|| ending.filter(|_| found == 1).map(|_| Shape::Acquired))
            .unwrap_or(Shape::Refused)
    }
}

impl Returned {
    const REFUSED: &'static str =
        "Held may be returned only bare, in an Option, or as the Ok of a Result, and never as a parameter";

    pub fn classified(output: &ReturnType) -> syn::Result<Returned> {
        match Shape::judged(output) {
            Shape::Plain => Ok(Returned::Plain(output.clone())),
            Shape::Acquired => Ok(Returned::Acquired(Self::released(output))),
            Shape::Refused => Err(Error::new_spanned(output, Self::REFUSED)),
        }
    }

    pub fn refuse_held_parameter(tree: &Type) -> syn::Result<()> {
        let mut names = HeldNames(Vec::new());
        names.visit_type(tree);
        names.0.first().map_or(Ok(()), |name| Err(Error::new_spanned(name, Self::REFUSED)))
    }

    pub fn filler_output(&self) -> &ReturnType {
        match self {
            Returned::Plain(output) | Returned::Acquired(output) => output,
        }
    }

    pub fn counting(&self, span: Span) -> Ident {
        match self {
            Returned::Plain(_) => Ident::new("counted_call", span),
            Returned::Acquired(_) => Ident::new("counted_acquire", span),
        }
    }

    fn tree(output: &ReturnType) -> Option<&Type> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, tree) => Some(tree),
        }
    }

    fn released(output: &ReturnType) -> ReturnType {
        let mut unheld = output.clone();
        Released.visit_return_type_mut(&mut unheld);
        unheld
    }
}
