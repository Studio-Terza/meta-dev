mod tokens;

use syntax::Spanned;
use syntax::{ExprPath, ExprStruct, Macro, QSelf, Type, TypeParamBound};

use graph::{Building, Domain, Member, Mention, PathUse, Region, Resolved};
use lint_layout::Position;

use crate::namespace::UsePath;

use super::Lowering;
use super::places::PlaceId;
use super::quotes::QuoteRow;
use super::seat::{Built, Footing, Located};
use lint_sweep::Address;
use tokens::TokenPaths;

#[derive(Debug)]
pub(super) struct UseRow {
    located: Located,
    written: UsePath,
    mention: Mention,
    region: Region,
    place: Option<PlaceId>,
}

impl UseRow {
    fn domain(&self) -> Domain {
        match self.mention {
            Mention::Invoked => Domain::Macros,
            Mention::Expression | Mention::InMacro | Mention::Interpolated | Mention::Dispatched | Mention::Typed => {
                Domain::Types
            }
        }
    }

    fn kept<'resolved>(&self, resolved: &'resolved Resolved) -> Option<&'resolved Resolved> {
        match self.mention {
            Mention::Invoked | Mention::Dispatched => Some(resolved),
            Mention::Expression | Mention::InMacro | Mention::Interpolated | Mention::Typed => {
                self.written.tail().first().map(|_| resolved).or_else(|| resolved.known())
            }
        }
    }
}

impl Built for UseRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let resolving = footing.resolving();
        let scope = self.located.scope();
        let direct = resolving.path(scope, &self.written, self.domain());
        let head = direct
            .unresolved()
            .and_then(|_| Some(resolving.path(scope, &self.written.parent()?, Domain::Types)))
            .zip(self.written.tail().last());
        let aliased = head.as_ref().and_then(|(parent, last)| building.aliased(parent)?.joined(last));
        let member = head
            .as_ref()
            .and_then(|(parent, last)| Some(Resolved::Member(Member::new(parent.crate_item()?, (*last).clone()))));
        let valued = || {
            direct.unresolved().and(Some(self.domain())).filter(|domain| *domain == Domain::Types)?;
            Some(resolving.path(scope, &self.written, Domain::Values)).filter(|resolved| resolved.known().is_some())
        };
        let resolved = aliased.or(member).or_else(valued).unwrap_or(direct);
        let written = self.written.written();
        let named = self.kept(&resolved).map(|_| ()).map(|()| resolved);
        let seat = footing.places().seat(self.region.joined(footing.outer()), self.place);
        let line = self.located.position().line();
        let used = named.map(|kept| PathUse::new(written, kept, self.mention, seat));
        used.into_iter().fold(building, |mut built, found| {
            built.path_use(footing.file(), line, found);
            built
        })
    }
}
#[derive(Debug)]
struct Dispatch;

impl Dispatch {
    const OWNED_MEMBER: usize = 2;

    fn objects(ty: &Type) -> Vec<UsePath> {
        match ty {
            Type::TraitObject(object) => object.bounds.iter().filter_map(Self::traited).collect(),
            _ => Vec::new(),
        }
    }

    fn traited(bound: &TypeParamBound) -> Option<UsePath> {
        match bound {
            TypeParamBound::Trait(traited) => Some(UsePath::from(&traited.path)),
            _ => None,
        }
    }

    fn qualified(qself: &QSelf, path: &syntax::Path) -> Option<UsePath> {
        let segments: syntax::Punctuated<syntax::PathSegment, syntax::Token![::]> =
            path.segments.iter().take(qself.position).cloned().collect();
        let present = segments.first().map(|_| ());
        present.map(|()| UsePath::from(&syntax::Path { leading_colon: path.leading_colon, segments }))
    }
}

#[derive(Debug)]
struct Typed;

impl Typed {
    fn path(ty: &Type) -> Option<&syntax::Path> {
        match ty {
            Type::Path(inner) => inner.qself.as_ref().map_or(Some(&inner.path), |_| None),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Qualified;

impl Qualified {
    fn written(expression: &ExprPath) -> Option<UsePath> {
        expression
            .qself
            .as_ref()
            .map_or_else(|| Some(UsePath::from(&expression.path)), |qself| Self::under_type(qself, &expression.path))
    }

    fn under_type(qself: &QSelf, path: &syntax::Path) -> Option<UsePath> {
        let trait_named = qself.position.checked_sub(1);
        let own = trait_named.map_or(Some(&*qself.ty), |_| None);
        own.and_then(Self::type_path).map(|inner| UsePath::from(inner).extended(&UsePath::from(path)))
    }

    fn type_path(ty: &Type) -> Option<&syntax::Path> {
        match ty {
            Type::Path(inner) => Some(&inner.path),
            _ => None,
        }
    }
}

impl Lowering<'_> {
    pub(super) fn path_used(&mut self, expression: &ExprPath) {
        let position = expression.path.segments.first().map(|first| Position::from(first.ident.span()));
        let called = self.calling.take().filter(|at| *at == Address::from(expression));
        let tabled = called.map_or(Some(()), |_| None).and(self.tables.last()).map(|_| ());
        let dispatched = tabled.map_or_default(|()| self.dispatched(expression));
        dispatched.into_iter().zip(position).fold(&mut *self, |lowering, (path, at)| {
            lowering.used(path, at, Mention::Dispatched);
            lowering
        });
        let found = Qualified::written(expression).zip(position);
        found.into_iter().fold(self, |lowering, (path, at)| {
            lowering.used(path, at, Mention::Expression);
            lowering
        });
    }

    pub(super) fn struct_used(&mut self, literal: &ExprStruct) {
        let position = literal.path.segments.first().map(|first| Position::from(first.ident.span()));
        let written = literal.qself.as_ref().map_or(Some(&literal.path), |_| None).zip(position);
        written.into_iter().fold(self, |lowering, (path, at)| {
            lowering.used(UsePath::from(path), at, Mention::Expression);
            lowering
        });
    }

    pub(super) fn types_used(&mut self, ty: &Type) {
        let position = Position::from(ty.span());
        let typed =
            self.context.in_body().and_then(|_| Typed::path(ty)).map(|path| (UsePath::from(path), Mention::Typed));
        let traits = Dispatch::objects(ty).into_iter().map(|path| (path, Mention::Dispatched));
        traits.chain(typed).fold(self, |lowering, (path, mention)| {
            lowering.used(path, position, mention);
            lowering
        });
    }

    fn dispatched(&self, expression: &ExprPath) -> Vec<UsePath> {
        let member = expression.path.segments.last().map(|last| UsePath::from(&last.ident));
        let qualified = expression.qself.as_ref().and_then(|qself| Dispatch::qualified(qself, &expression.path));
        let head =
            expression.path.segments.first().filter(|_| expression.path.segments.len() == Dispatch::OWNED_MEMBER);
        let bounded: Vec<UsePath> = head.map_or_default(|first| {
            self.generics.traits_of(&lint_layout::Name::from(&first.ident)).into_iter().cloned().collect()
        });
        let traits = qualified.into_iter().chain(bounded);
        member.map_or_default(|named| traits.map(|traited| traited.extended(&named)).collect())
    }

    pub(super) fn macro_used(&mut self, invoked: &Macro) {
        let position = invoked
            .path
            .segments
            .last()
            .map_or_else(|| Position::from(invoked.path.span()), |last| Position::from(last.ident.span()));
        let law = self.law_row(invoked);
        self.lowered.rows.laws.extend(law);
        let quotes = QuoteRow::found(self.source, invoked, &self.here);
        self.lowered.rows.quotes.extend(quotes);
        self.used(UsePath::from(&invoked.path), position, Mention::Invoked);
        let plain = TokenPaths::within(invoked.tokens.clone()).into_iter().map(|spelled| (spelled, Mention::InMacro));
        let interpolated =
            TokenPaths::interpolated(invoked).into_iter().map(|spelled| (spelled, Mention::Interpolated));
        plain.chain(interpolated).fold(self, |lowering, (spelled, mention)| {
            spelled.handed(|path, at| lowering.used(path, at, mention));
            lowering
        });
    }

    fn used(&mut self, written: UsePath, position: Position, mention: Mention) {
        let row = UseRow {
            located: self.here.located(position),
            written,
            mention,
            region: self.context.region(),
            place: self.context.place(),
        };
        self.lowered.rows.uses.push(row);
    }
}
