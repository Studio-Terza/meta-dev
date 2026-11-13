mod tokens;

use syn::spanned::Spanned;
use syn::{ExprPath, Macro, QSelf, Type};

use code_model::{Building, Domain, Member, Mention, PathUse, Region, Resolved};
use lint_layout::Position;

use crate::namespace::UsePath;

use super::Lowering;
use super::places::PlaceId;
use super::seat::{Built, Footing, Located};
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
            Mention::Expression | Mention::InMacro => Domain::Types,
        }
    }

    fn kept<'resolved>(&self, resolved: &'resolved Resolved) -> Option<&'resolved Resolved> {
        match self.mention {
            Mention::Invoked => Some(resolved),
            Mention::Expression | Mention::InMacro => {
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
        let member = head.as_ref().and_then(|(parent, _)| Some(Resolved::Member(Member::from(parent.crate_item()?))));
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
struct Qualified;

impl Qualified {
    fn written(expression: &ExprPath) -> Option<UsePath> {
        expression
            .qself
            .as_ref()
            .map_or_else(|| Some(UsePath::from(&expression.path)), |qself| Self::under_type(qself, &expression.path))
    }

    fn under_type(qself: &QSelf, path: &syn::Path) -> Option<UsePath> {
        let trait_named = qself.position.checked_sub(1);
        let own = trait_named.map_or(Some(&*qself.ty), |_| None);
        own.and_then(Self::type_path).map(|inner| UsePath::from(inner).extended(&UsePath::from(path)))
    }

    fn type_path(ty: &Type) -> Option<&syn::Path> {
        match ty {
            Type::Path(inner) => Some(&inner.path),
            _ => None,
        }
    }
}

impl Lowering<'_> {
    pub(super) fn path_used(&mut self, expression: &ExprPath) {
        let position = expression.path.segments.first().map(|first| Position::from(first.ident.span()));
        let found = Qualified::written(expression).zip(position);
        found.into_iter().fold(self, |lowering, (path, at)| {
            lowering.used(path, at, Mention::Expression);
            lowering
        });
    }

    pub(super) fn macro_used(&mut self, invoked: &Macro) {
        let position = invoked
            .path
            .segments
            .last()
            .map_or_else(|| Position::from(invoked.path.span()), |last| Position::from(last.ident.span()));
        self.used(UsePath::from(&invoked.path), position, Mention::Invoked);
        TokenPaths::within(invoked.tokens.clone()).into_iter().fold(self, |lowering, spelled| {
            spelled.handed(|path, at| lowering.used(path, at, Mention::InMacro));
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
