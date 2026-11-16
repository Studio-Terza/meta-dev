mod boundary;

use std::fmt;

use syntax::{Around, Macro, Node, Walk};

use lint_core::{ByKind, Descend, Facts, Kind, Macros, SourceFile, Unwalked, Violation, Watch, Watching};
use lint_layout::Name;

pub use boundary::Member;
use boundary::{Index, Muting};

pub trait Gathers<Part> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>>;

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
        Walk::file(&mut Sweep::sweeping(Some(file), gathering.watching().into_iter().collect()), file.syntax());
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
        let kinds: Vec<&[Kind]> = watchers.iter().map(|watcher| watcher.kinds()).collect();
        let mut heard = Index::listening(&kinds);
        let listening = ByKind::filled(|kind| heard.remove(&kind).unwrap_or_default());
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
        let Sweeping { listening, watchers, muting, .. } = self;
        muting.entering();
        listening.get(kind).iter().fold(Entered::default(), |entered, index| {
            let watcher = muting.heard(*index).and_then(|heard| heard.entry_mut(watchers));
            entered.chosen(muting, *index, watcher.map(|found| enter(&mut **found)))
        })
    }

    fn left(&mut self, kind: Kind, leave: impl Fn(&mut Watching<'run>)) {
        let Sweeping { listening, watchers, muting, .. } = self;
        let heard = listening.get(kind);
        heard.iter().filter_map(|index| muting.heard(*index)).for_each(|index| {
            index.entry_mut(watchers).into_iter().for_each(|watcher| leave(&mut **watcher));
        });
        muting.leaving(heard);
    }

    fn around_kind(
        &mut self,
        kind: Kind,
        enter: impl Fn(&mut Watching<'run>) -> Descend,
        descend: &mut dyn FnMut(&mut Self),
        leave: impl Fn(&mut Watching<'run>),
    ) {
        self.entered(kind, enter);
        self.muting.awake().into_iter().for_each(|_| descend(self));
        self.left(kind, leave);
    }

    fn expanded(&mut self, entered: &Entered, invocation: &Macro) {
        let statements = self.file.map_or_default(|file| Macros::expanded(file, invocation));
        let saved = self.muting.hushed(&entered.expanding);
        statements.iter().for_each(|statement| Walk::stmt(self, statement));
        self.muting.restored(saved);
    }
}

#[derive(Debug, Default)]
struct Entered {
    expanding: Vec<Index>,
}

impl Entered {
    fn chosen(mut self, muting: &mut Muting, index: Index, descend: Option<Descend>) -> Entered {
        let kept = descend.filter(|chosen| *chosen != Descend::Into);
        kept.filter(|chosen| *chosen != Descend::Unfold).into_iter().for_each(|_| muting.silence(index));
        self.expanding.extend(kept.filter(|chosen| *chosen != Descend::Over).map(|_| index));
        self
    }
}

impl Sweeping<'_, '_> {
    fn macro_around(&mut self, node: &Macro, descend: &mut dyn FnMut(&mut Self)) {
        self.file.map_or_default(|file| Macros::pin(file, node));
        let facts = Facts::new(node);
        let entered = self.entered(Kind::Macro, |watcher| watcher.on_macro(node, &facts));
        self.muting.awake().into_iter().for_each(|_| descend(self));
        entered.expanding.first().into_iter().for_each(|_| self.expanded(&entered, node));
        self.left(Kind::Macro, |watcher| watcher.off_macro(node, &facts));
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
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        self.0.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() }).collect::<Vec<_>>()
    }

    fn gathered(self) -> Vec<Violation<'file>> {
        self.0.into_iter().flat_map(Finds::found).collect()
    }
}
