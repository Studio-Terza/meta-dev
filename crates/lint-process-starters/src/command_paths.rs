use std::path::Path;

use lint_core::{
    Address, Around, Contribute, Contributed, Contributing, Descend, Facts, Gate, Joins, Kind, Site, SourceFile,
};
use lint_layout::{LineNumber, Name, Word};
use syntax::{Ident, ItemMod, ItemUse, Macro, Spanned, UseLeaf};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    #[default]
    OutsidePorts,
    Everywhere,
}

#[derive(Debug)]
pub struct CommandPaths;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ported;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Module {
    Root,
    Inline(Address),
}

#[derive(Clone, Copy, Debug)]
struct Enclosing {
    module: Module,
    ported: Option<Ported>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Bound {
    Command(Name),
    Process(Name),
}

#[derive(Debug)]
struct Import {
    module: Module,
    bound: Bound,
}

#[derive(Debug)]
struct Spelled {
    module: Module,
    line: LineNumber,
    segments: Vec<Name>,
}

#[derive(Debug, Default)]
struct Spotted {
    imports: Vec<Import>,
    spelled: Vec<Spelled>,
}

#[derive(Debug)]
struct Scan {
    scope: Scope,
}

impl CommandPaths {
    const PROCESS: Word<'static> = Word::new("process");
    const COMMAND: Word<'static> = Word::new("Command");
    const NEW: Word<'static> = Word::new("new");
    const STD: Word<'static> = Word::new("std");
    const OWN: Word<'static> = Word::new("self");

    #[must_use]
    pub fn found(file: &SourceFile, scope: Scope) -> Vec<Site<'_>> {
        let path = AsRef::<Path>::as_ref(file.head());
        let admitted = scope.inside(Gate::PORTS.admits(file.head(), Ported));
        admitted.map_or_default(|_| {
            Contributing::new(Scan { scope }).concluded(file, |_, _, spotted: Spotted| spotted.starts(path))
        })
    }
}

impl Scope {
    fn inside(self, ported: Option<Ported>) -> Option<Scope> {
        match self {
            Scope::Everywhere => Some(self),
            Scope::OutsidePorts => ported.map_or(Some(self), |_| None),
        }
    }
}

impl Joins for Spotted {
    fn combine(mut self, later: Spotted) -> Spotted {
        self.imports.extend(later.imports);
        self.spelled.extend(later.spelled);
        self
    }
}

impl Spotted {
    fn starts(self, path: &Path) -> Vec<Site<'_>> {
        let Spotted { imports, spelled } = self;
        spelled.iter().filter_map(|spelling| spelling.started(&imports)).map(|line| Site::new(path, line)).collect()
    }
}

impl Spelled {
    fn started(&self, imports: &[Import]) -> Option<LineNumber> {
        let mut reversed = self.segments.iter().rev();
        reversed.next().filter(|last| **last == Name::from(CommandPaths::NEW))?;
        let owner = reversed.next()?;
        let before = reversed.next();
        let bound: Vec<&Bound> =
            imports.iter().filter(|import| import.module == self.module).map(|import| &import.bound).collect();
        let imported = bound.iter().find(|held| ***held == Bound::Command(owner.clone())).map(|_| ());
        let qualified = before
            .filter(|_| *owner == Name::from(CommandPaths::COMMAND))
            .filter(|module| {
                **module == Name::from(CommandPaths::PROCESS) || bound.contains(&&Bound::Process((*module).clone()))
            })
            .map(|_| ());
        imported.or(qualified).map(|()| self.line)
    }
}

impl Bound {
    fn of_leaf(leaf: &UseLeaf<'_>) -> Option<Bound> {
        let parent = Name::from(*leaf.path().last()?);
        let end = leaf.end();
        let named = end.named().map(|named| (&named.ident, &named.ident));
        let spelled = named.or_else(|| end.renamed().map(|renamed| (&renamed.ident, &renamed.rename)));
        spelled.map_or_else(
            || (parent == Name::from(CommandPaths::PROCESS)).then(|| Bound::Command(Name::from(CommandPaths::COMMAND))),
            |(original, bound)| Bound::named(&parent, original, bound),
        )
    }

    fn named(parent: &Name, original: &Ident, bound: &Ident) -> Option<Bound> {
        let original = Name::from(original);
        let bound = Name::from(bound);
        let process = Name::from(CommandPaths::PROCESS);
        let in_process = (*parent == process).then_some(());
        let command = in_process.filter(|()| original == Name::from(CommandPaths::COMMAND));
        let itself = in_process.filter(|()| original == Name::from(CommandPaths::OWN));
        let from_std = (*parent == Name::from(CommandPaths::STD)).then_some(()).filter(|()| original == process);
        let module = itself.or(from_std).map(|()| Bound::Process(Bound::spelled(bound.clone(), &process)));
        command.map(|()| Bound::Command(bound)).or(module)
    }

    fn spelled(bound: Name, process: &Name) -> Name {
        Some(bound).filter(|name| *name != Name::from(CommandPaths::OWN)).unwrap_or_else(|| process.clone())
    }
}

impl Scan {
    fn enclosing<'up>(around: &Around<'up, Vec<Enclosing>>) -> impl Iterator<Item = &'up Enclosing> {
        around.nearest().flat_map(|open| open.iter())
    }

    fn module(around: &Around<'_, Vec<Enclosing>>) -> Module {
        Self::enclosing(around).next().map_or(Module::Root, |enclosing| enclosing.module)
    }

    fn ported(around: &Around<'_, Vec<Enclosing>>) -> Option<Ported> {
        Self::enclosing(around).find_map(|enclosing| enclosing.ported)
    }
}

impl Contribute for Scan {
    type Open = Vec<Enclosing>;
    type Part = Spotted;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemUse, Kind::Path, Kind::Macro]
    }

    fn scopes(&self) -> &'static [lint_core::Scope] {
        const { &[lint_core::Scope::new(Kind::ItemMod)] }
    }

    fn on_item_mod(
        &self,
        module: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        around: &Around<'_, Vec<Enclosing>>,
    ) -> Contributed<Vec<Enclosing>, Spotted> {
        let ported = Gate::PORTS.names(module, Ported).or_else(|| Scan::ported(around));
        let opened = Enclosing { module: Module::Inline(facts.address()), ported };
        Contributed::new(vec![opened], Spotted::default(), Descend::Into)
    }

    fn on_item_use(
        &self,
        item: &ItemUse,
        _: &Facts<'_, ItemUse>,
        around: &Around<'_, Vec<Enclosing>>,
    ) -> Contributed<Vec<Enclosing>, Spotted> {
        let module = Scan::module(around);
        let imports = UseLeaf::of_tree(&item.tree)
            .iter()
            .filter_map(Bound::of_leaf)
            .map(|bound| Import { module, bound })
            .collect();
        Contributed::new(Vec::new(), Spotted { imports, spelled: Vec::new() }, Descend::Over)
    }

    fn on_path(
        &self,
        path: &syntax::Path,
        _: &Facts<'_, syntax::Path>,
        around: &Around<'_, Vec<Enclosing>>,
    ) -> Contributed<Vec<Enclosing>, Spotted> {
        let spelled = self.scope.inside(Scan::ported(around)).map(|_| Spelled {
            module: Scan::module(around),
            line: LineNumber::from(path.span()),
            segments: path.segments.iter().map(|segment| Name::from(&segment.ident)).collect(),
        });
        Contributed::parted(Spotted { imports: Vec::new(), spelled: spelled.into_iter().collect() })
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _: &Around<'_, Vec<Enclosing>>,
    ) -> Contributed<Vec<Enclosing>, Spotted> {
        Contributed::descending(Descend::Expand)
    }
}
