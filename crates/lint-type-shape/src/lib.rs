use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Address, Descend, Facts, Finds, Fold, Folding, Kind, Visit};
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
        Box::new(Folding::from(TypeShapes { path: AsRef::<Path>::as_ref(file.head()) }))
    }
}

#[derive(Debug)]
struct TypeShapes<'scan> {
    path: &'scan Path,
}

#[derive(Debug, Default)]
struct Shapes<'scan> {
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

impl<'scan> From<Shapes<'scan>> for Vec<Violation<'scan>> {
    fn from(shapes: Shapes<'scan>) -> Vec<Violation<'scan>> {
        shapes.violations
    }
}

impl<'scan> Shapes<'scan> {
    fn rooted(self, roots: impl IntoIterator<Item = Root<'scan>>) -> Shapes<'scan> {
        Shapes { roots: self.roots.into_iter().chain(roots).collect(), ..self }
    }

    fn flagged(self, flagged: Option<Violation<'scan>>) -> Shapes<'scan> {
        Shapes { violations: self.violations.into_iter().chain(flagged).collect(), ..self }
    }

    fn entered(self, at: Address) -> Shapes<'scan> {
        let (reached, roots): (Vec<Root<'scan>>, Vec<Root<'scan>>) =
            self.roots.into_iter().partition(|root| root.at == at);
        let opened: Vec<Open> = reached.iter().map(|root| Open { at: root.at, span: root.span }).collect();
        let violations = self.violations.into_iter().chain(reached.into_iter().filter_map(|root| root.first)).collect();
        Shapes { violations, roots, open: self.open.into_iter().chain(opened).collect() }
    }

    fn left(self, ty: &Type) -> Shapes<'scan> {
        let at = Address::from(ty);
        let mut open = self.open;
        open.pop_if(|last| last.at == at);
        Shapes { open, ..self }
    }
}

impl<'scan> TypeShapes<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn roots(&self, fields: &Fields) -> Vec<Root<'scan>> {
        fields.iter().map(|field| Root::new(&field.ty, self.bool_field(field))).collect()
    }

    fn found(&self, shapes: Shapes<'scan>, message: Option<Message>) -> Shapes<'scan> {
        let at = shapes.open.last().map(|open| open.span);
        let found = at.zip(message).map(|(span, finding)| self.violation(span, finding));
        shapes.flagged(found)
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

impl<'scan> Fold for TypeShapes<'scan> {
    type Found = Shapes<'scan>;

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

    fn on_item_mod(&self, found: Shapes<'scan>, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Shapes<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, found: Shapes<'scan>, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Shapes<'scan>> {
        Visit::new(found, facts.inherent().map_or(Descend::Over, |_| Descend::Into))
    }

    fn on_item_struct(
        &self,
        found: Shapes<'scan>,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
    ) -> Visit<Shapes<'scan>> {
        Visit::descending(found.rooted(self.roots(&item.fields)))
    }

    fn on_item_enum(&self, found: Shapes<'scan>, item: &ItemEnum, _: &Facts<'_, ItemEnum>) -> Visit<Shapes<'scan>> {
        Visit::descending(found.rooted(item.variants.iter().flat_map(|variant| self.roots(&variant.fields))))
    }

    fn on_item_type(&self, found: Shapes<'scan>, item: &ItemType, _: &Facts<'_, ItemType>) -> Visit<Shapes<'scan>> {
        Visit::descending(found.rooted([Root::new(&item.ty, None)]))
    }

    fn on_macro(&self, found: Shapes<'scan>, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Shapes<'scan>> {
        Visit::new(found, Descend::Expand)
    }

    fn on_signature(
        &self,
        found: Shapes<'scan>,
        signature: &Signature,
        _: &Facts<'_, Signature>,
    ) -> Visit<Shapes<'scan>> {
        let typed = signature.inputs.iter().filter_map(Syntax::typed).map(|typed| &*typed.ty);
        Visit::descending(found.rooted(typed.chain(Syntax::returned(&signature.output)).map(|ty| Root::new(ty, None))))
    }

    fn on_type(&self, found: Shapes<'scan>, ty: &Type, _: &Facts<'_, Type>) -> Visit<Shapes<'scan>> {
        Visit::descending(found.entered(Address::from(ty)))
    }

    fn off_type(&self, found: Shapes<'scan>, ty: &Type, _: &Facts<'_, Type>) -> Shapes<'scan> {
        found.left(ty)
    }

    fn on_type_tuple(&self, found: Shapes<'scan>, tuple: &TypeTuple, _: &Facts<'_, TypeTuple>) -> Visit<Shapes<'scan>> {
        Visit::descending(self.found(found, tuple.elems.first().map(|_| TypeShape::TUPLE)))
    }

    fn on_type_path(&self, found: Shapes<'scan>, path: &TypePath, _: &Facts<'_, TypePath>) -> Visit<Shapes<'scan>> {
        Visit::descending(self.found(found, TypeShape::string_error(path)))
    }
}
