use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Address, Descend, Facts, Finds, Kind, Watch};
use std::path::Path;
use syntax::Span;
use syntax::Spanned;

use syntax::{
    Field, Fields, GenericArgument, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Macro, Signature, Type,
    TypePath, TypeTuple,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct TypeShape;

impl TypeShape {
    const TUPLE: Message = Message::new(
        Problem::new("tuple in a signature or field"),
        Fix::new("values that travel together are a struct"),
    );

    fn string_error(path: &TypePath) -> Option<Message> {
        path.path
            .segments
            .last()
            .filter(|segment| segment.ident == "Result")
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|arguments| arguments.args.iter().nth(1))
            .filter(|argument| {
                matches!(
                    argument,
                    GenericArgument::Type(Type::Path(error))
                        if error.path.segments.last().is_some_and(|segment| segment.ident == "String")
                )
            })
            .map(|_| {
                Message::new(Problem::new("Result with a String error"), Fix::new("errors are an enum with thiserror"))
            })
    }
}

impl SweptLint for TypeShape {
    const HEADLINE: Headline =
        Headline::new("types carry their meaning: no bool fields, no tuples in signatures, no String errors");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(TypeShapes {
            path: AsRef::<Path>::as_ref(file.head()),
            violations: Vec::new(),
            roots: Vec::new(),
            open: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct TypeShapes<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
    roots: Vec<Root<'scan>>,
    open: Vec<Open>,
}

#[derive(Debug)]
struct Root<'scan> {
    at: Address,
    span: Span,
    first: Option<Violation<'scan>>,
}

impl<'scan> Root<'scan> {
    fn new(ty: &Type, first: Option<Violation<'scan>>) -> Root<'scan> {
        Root { at: Address::from(ty), span: ty.span(), first }
    }
}

#[derive(Clone, Copy, Debug)]
struct Open {
    at: Address,
    span: Span,
}

impl<'scan> TypeShapes<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn check_fields(&mut self, fields: &Fields) {
        fields.iter().for_each(|field| {
            let first = self.bool_field(field);
            self.roots.push(Root::new(&field.ty, first));
        });
    }

    fn found(&mut self, message: Option<Message>) {
        let at = self.open.last().map(|open| open.span);
        let found = at.zip(message).map(|(span, finding)| self.violation(span, finding));
        self.violations.extend(found);
    }

    fn bool_field(&self, field: &Field) -> Option<Violation<'scan>> {
        matches!(
            &field.ty,
            Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "bool")
        )
        .then(|| {
            self.violation(
                field.ident.as_ref().map_or_else(|| field.ty.span(), Ident::span),
                Message::new(
                    Problem::new("bool field"),
                    Fix::new("a stored flag is a stored mode, make it an enum or a type parameter"),
                ),
            )
        })
    }
}

impl<'ast> Watch<'ast> for TypeShapes<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemStruct,
            Kind::ItemEnum,
            Kind::ItemType,
            Kind::Macro,
            Kind::Signature,
            Kind::Type,
            Kind::TypeTuple,
            Kind::TypePath,
        ]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_impl(&mut self, _: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        facts.inherent().map_or(Descend::Over, |_| Descend::Into)
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct, _: &Facts<'_, ItemStruct>) -> Descend {
        self.check_fields(&item.fields);
        Descend::Into
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum, _: &Facts<'_, ItemEnum>) -> Descend {
        item.variants.iter().for_each(|variant| self.check_fields(&variant.fields));
        Descend::Into
    }

    fn on_item_type(&mut self, item: &'ast ItemType, _: &Facts<'_, ItemType>) -> Descend {
        self.roots.push(Root::new(&item.ty, None));
        Descend::Into
    }

    fn on_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        Descend::Expand
    }

    fn on_signature(&mut self, signature: &'ast Signature, _: &Facts<'_, Signature>) -> Descend {
        signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .map(|typed| &*typed.ty)
            .chain(Syntax::returned(&signature.output))
            .for_each(|ty| self.roots.push(Root::new(ty, None)));
        Descend::Into
    }

    fn on_type(&mut self, ty: &'ast Type, _: &Facts<'_, Type>) -> Descend {
        let at = Address::from(ty);
        let (mut reached, waiting): (Vec<Root<'_>>, Vec<Root<'_>>) =
            std::mem::take(&mut self.roots).into_iter().partition(|root| root.at == at);
        self.roots = waiting;
        self.violations.extend(reached.iter_mut().filter_map(|root| root.first.take()));
        self.open.extend(reached.iter().map(|root| Open { at: root.at, span: root.span }));
        Descend::Into
    }

    fn off_type(&mut self, ty: &'ast Type, _: &Facts<'_, Type>) {
        let at = Address::from(ty);
        self.open.pop_if(|open| open.at == at);
    }

    fn on_type_tuple(&mut self, tuple: &'ast TypeTuple, _: &Facts<'_, TypeTuple>) -> Descend {
        self.found(tuple.elems.first().map(|_| TypeShape::TUPLE));
        Descend::Into
    }

    fn on_type_path(&mut self, path: &'ast TypePath, _: &Facts<'_, TypePath>) -> Descend {
        self.found(TypeShape::string_error(path));
        Descend::Into
    }
}

impl<'file> Finds<'file> for TypeShapes<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}
