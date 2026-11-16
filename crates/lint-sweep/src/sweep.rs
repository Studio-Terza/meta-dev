mod boundary;

use std::fmt;

use syntax::{Around, Macro, Node, Walk};

use lint_core::{ByKind, Descend, Facts, Kind, Macros, SourceFile, Unwalked, Violation, Walks, Watch, Watching};
use lint_layout::Name;

pub use boundary::Member;
use boundary::{Index, Muting};

pub trait Gathers<Part> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>>;

    fn gathered(self) -> Part;
}

pub trait Finds<'file>: for<'ast> Watch<'ast> + fmt::Debug {
    fn found(self: Box<Self>) -> Vec<Violation<'file>>;
}

pub trait Sorts<'file>: for<'ast> Watch<'ast> + fmt::Debug {
    fn sorted(self: Box<Self>) -> Vec<Vec<Violation<'file>>>;
}

#[derive(Debug)]
pub struct Sweep;

impl Sweep {
    pub fn over_all<'file, 'run>(file: &'file SourceFile, groups: &mut [Vec<Box<dyn Finds<'file> + 'run>>]) {
        let watching: Vec<&mut Watching<'_>> = groups
            .iter_mut()
            .flat_map(|group| group.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() }))
            .collect();
        Walk::file(&mut Sweep::sweeping(Some(file), watching), file.syntax());
    }

    pub fn over_leads<'file, 'run>(
        file: &'file SourceFile,
        finds: &mut [Box<dyn Finds<'file> + 'run>],
        sorts: &mut [Box<dyn Sorts<'file> + 'run>],
    ) {
        let found = finds.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() });
        let sorting = sorts.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() });
        Walk::file(&mut Sweep::sweeping(Some(file), found.chain(sorting).collect()), file.syntax());
    }

    pub fn gathered<Part>(file: &SourceFile, mut gathering: impl Gathers<Part>) -> Part {
        Walk::file(&mut Sweep::sweeping(Some(file), gathering.watching()), file.syntax());
        gathering.gathered()
    }

    pub fn alone<'run>(watcher: &'run mut Watching<'run>) -> Vec<&'run mut Watching<'run>> {
        vec![watcher]
    }

    pub fn watching<'run>(file: &SourceFile, watchers: Vec<&'run mut Watching<'run>>) {
        Walk::file(&mut Sweep::sweeping(Some(file), watchers), file.syntax());
    }

    pub fn once<'run>(_: Unwalked, file: &SourceFile, watchers: Vec<&'run mut Watching<'run>>) {
        Walk::file(&mut Sweep::sweeping(Some(file), watchers), file.syntax());
    }

    pub fn expanded_item(item: &syntax::Item, watcher: &mut Watching<'_>) {
        Walk::item_node(&mut Sweep::sweeping(None, vec![watcher]), item);
    }

    fn sweeping<'file, 'run>(
        file: Option<&'file SourceFile>,
        watchers: Vec<&'run mut Watching<'run>>,
    ) -> Sweeping<'file, 'run> {
        file.map(SourceFile::walks).into_iter().for_each(Walks::walked);
        let kinds: Vec<&[Kind]> = watchers.iter().map(|watcher| watcher.kinds()).collect();
        let listening = ByKind::filled(|kind| Index::listening(&kinds, kind));
        let muting = Muting::new(watchers.len());
        Sweeping { file, watchers, listening, muting }
    }

    #[must_use]
    pub fn found<'file, 'run>(file: &'file SourceFile, watcher: Box<dyn Finds<'file> + 'run>) -> Vec<Violation<'file>> {
        let mut watchers = [vec![watcher]];
        Sweep::over_all(file, &mut watchers);
        let [only] = watchers;
        only.into_iter().flat_map(Finds::found).collect()
    }
}

struct Sweeping<'file, 'run> {
    file: Option<&'file SourceFile>,
    watchers: Vec<&'run mut Watching<'run>>,
    listening: ByKind<Vec<Index>>,
    muting: Muting,
}

impl<'run> Sweeping<'_, 'run> {
    fn entered(&mut self, kind: Kind, enter: impl Fn(&mut Watching<'run>) -> Descend) -> Entered {
        self.muting.entering();
        let called: Vec<Index> =
            self.listening.get(kind).iter().filter_map(|index| self.muting.listening(*index)).collect();
        let chosen: Vec<(Index, Descend)> = called
            .iter()
            .filter_map(|index| Some((*index, enter(&mut **index.entry_mut(&mut self.watchers)?))))
            .filter(|(_, descend)| *descend != Descend::Into)
            .collect();
        chosen
            .iter()
            .filter(|(_, descend)| *descend != Descend::Unfold)
            .for_each(|(index, _)| self.muting.silence(*index));
        let expanding = chosen.iter().filter(|(_, descend)| *descend != Descend::Over).map(|(index, _)| *index);
        Entered { expanding: expanding.collect(), called }
    }

    fn left(&mut self, entered: &Entered, leave: impl Fn(&mut Watching<'run>)) {
        entered.called.iter().for_each(|index| {
            index.entry_mut(&mut self.watchers).into_iter().for_each(|watcher| leave(&mut **watcher));
        });
        self.muting.leaving(&entered.called);
    }

    fn around_kind(
        &mut self,
        kind: Kind,
        enter: impl Fn(&mut Watching<'run>) -> Descend,
        descend: &mut dyn FnMut(&mut Self),
        leave: impl Fn(&mut Watching<'run>),
    ) {
        let entered = self.entered(kind, enter);
        self.muting.awake().into_iter().for_each(|_| descend(self));
        self.left(&entered, leave);
    }

    fn expanded(&mut self, entered: &Entered, invocation: &Macro) {
        let statements = self.file.map_or_default(|file| Macros::expanded(file, invocation));
        let saved = self.muting.hushed(&entered.expanding);
        statements.iter().for_each(|statement| Walk::stmt(self, statement));
        self.muting.restored(saved);
    }
}

#[derive(Debug)]
struct Entered {
    expanding: Vec<Index>,
    called: Vec<Index>,
}

impl Sweeping<'_, '_> {
    fn macro_around(&mut self, node: &Macro, descend: &mut dyn FnMut(&mut Self)) {
        self.file.map_or_default(|file| Macros::pin(file, node));
        let facts = Facts::new(node);
        let entered = self.entered(Kind::Macro, |watcher| watcher.on_macro(node, &facts));
        self.muting.awake().into_iter().for_each(|_| descend(self));
        entered.expanding.first().into_iter().for_each(|_| self.expanded(&entered, node));
        self.left(&entered, |watcher| watcher.off_macro(node, &facts));
    }

    fn ident_around(&mut self, node: &syntax::Ident) {
        let heard = self.listening.get(Kind::Ident).first().map(|_| Name::from(node));
        heard.map_or_default(|name| {
            self.around_kind(
                Kind::Ident,
                |watcher| watcher.on_ident(node, &name),
                &mut |_| {},
                |watcher| watcher.off_ident(node),
            );
        });
    }
}

impl Around for Sweeping<'_, '_> {
    lint_core::sweep_nodes!();
}

#[derive(Debug)]
pub struct Watchers<'file>(Vec<Box<dyn Finds<'file> + 'file>>);

impl<'file> Watchers<'file> {
    #[must_use]
    pub fn none() -> Watchers<'file> {
        Watchers(Vec::new())
    }

    #[must_use]
    pub fn and(mut self, watcher: Box<dyn Finds<'file> + 'file>) -> Watchers<'file> {
        self.0.push(watcher);
        self
    }
}

impl<'file> Gathers<Vec<Violation<'file>>> for Watchers<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.0.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() }).collect()
    }

    fn gathered(self) -> Vec<Violation<'file>> {
        self.0.into_iter().flat_map(Finds::found).collect()
    }
}
