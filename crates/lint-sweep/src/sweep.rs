mod boundary;

use syn::Macro;
use syn::visit::{self, Visit};

use lint_core::{ByKind, Descend, Kind, Macros, SourceFile, Violation, Watch, Watching};

use boundary::{Index, Muting};

pub trait Gathers<Part> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>>;

    fn gathered(self) -> Part;
}

pub trait Finds<'file>: for<'ast> Watch<'ast> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>>;
}

#[derive(Debug)]
pub struct Sweep;

impl Sweep {
    pub fn over<'file>(file: &'file SourceFile, watchers: &mut [Box<dyn Finds<'file> + 'file>]) {
        let watching: Vec<&mut Watching<'_>> =
            watchers.iter_mut().map(|watcher| -> &mut Watching<'_> { watcher.as_mut() }).collect();
        Sweep::sweeping(Some(file), watching).visit_file(file.syntax());
    }

    pub fn through_file(file: &SourceFile, watcher: &mut Watching<'_>) {
        Sweep::sweeping(Some(file), vec![watcher]).visit_file(file.syntax());
    }

    pub fn gathered<Part>(file: &SourceFile, mut gathering: impl Gathers<Part>) -> Part {
        Sweep::sweeping(Some(file), gathering.watching()).visit_file(file.syntax());
        gathering.gathered()
    }

    pub fn alone<'run>(watcher: &'run mut Watching<'run>) -> Vec<&'run mut Watching<'run>> {
        vec![watcher]
    }

    pub fn watching<'run>(file: &SourceFile, watchers: Vec<&'run mut Watching<'run>>) {
        Sweep::sweeping(Some(file), watchers).visit_file(file.syntax());
    }

    pub fn through_block(file: &SourceFile, block: &syn::Block, watcher: &mut Watching<'_>) {
        Sweep::sweeping(Some(file), vec![watcher]).visit_block(block);
    }

    pub fn through_expr(expr: &syn::Expr, watcher: &mut Watching<'_>) {
        Sweep::sweeping(None, vec![watcher]).visit_expr(expr);
    }

    pub fn through_type(ty: &syn::Type, watcher: &mut Watching<'_>) {
        Sweep::sweeping(None, vec![watcher]).visit_type(ty);
    }

    fn sweeping<'file, 'run>(
        file: Option<&'file SourceFile>,
        watchers: Vec<&'run mut Watching<'run>>,
    ) -> Sweeping<'file, 'run> {
        let kinds: Vec<&[Kind]> = watchers.iter().map(|watcher| watcher.kinds()).collect();
        let listening = ByKind::filled(|kind| Index::listening(&kinds, kind));
        let muting = Muting::new(watchers.len());
        Sweeping { file, watchers, listening, muting }
    }

    #[must_use]
    pub fn found<'file>(file: &'file SourceFile, watcher: Box<dyn Finds<'file> + 'file>) -> Vec<Violation<'file>> {
        let mut watchers = [watcher];
        Sweep::over(file, &mut watchers);
        let [only] = watchers;
        only.found()
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

    fn around(
        &mut self,
        kind: Kind,
        enter: impl Fn(&mut Watching<'run>) -> Descend,
        descend: impl FnOnce(&mut Self),
        leave: impl Fn(&mut Watching<'run>),
    ) {
        let entered = self.entered(kind, enter);
        self.muting.awake().map(|_| descend).into_iter().for_each(|go| go(self));
        self.left(&entered, leave);
    }

    fn expanded(&mut self, entered: &Entered, invocation: &Macro) {
        let statements = self.file.map_or_default(|file| Macros::expanded(file, invocation));
        let saved = self.muting.hushed(&entered.expanding);
        statements.iter().for_each(|statement| self.visit_stmt(statement));
        self.muting.restored(saved);
    }
}

#[derive(Debug)]
struct Entered {
    expanding: Vec<Index>,
    called: Vec<Index>,
}

impl<'ast> Visit<'ast> for Sweeping<'_, '_> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        let entered = self.entered(Kind::Macro, |watcher| watcher.on_macro(node));
        self.muting.awake().into_iter().for_each(|_| visit::visit_macro(self, node));
        entered.expanding.first().into_iter().for_each(|_| self.expanded(&entered, node));
        self.left(&entered, |watcher| watcher.off_macro(node));
    }

    lint_core::sweep_visits!();
}
