use std::collections::BTreeMap;
use std::sync::mpsc::{self, Receiver, Sender};

use file_ports::Files;
use lint_core::{FileHead, Readout, SourceFile};
use lint_placement::Packages;

use super::boundary::Partition;
use super::protocol::{Assignment, Parsed, Reads, Reply};
use crate::source_files::located::{Located, Node};

#[derive(Debug)]
enum Order<'tree> {
    Parse(Vec<Node<'tree>>),
    Assemble(Assignment<'tree>),
}

impl<'tree> Order<'tree> {
    fn parsing(self) -> Vec<Node<'tree>> {
        match self {
            Order::Parse(nodes) => nodes,
            Order::Assemble(_) => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Reader<'tree> {
    hand: Partition,
    packages: &'tree Packages,
    orders: Receiver<Order<'tree>>,
    replies: Sender<Reply<'tree>>,
    heads: Sender<Vec<FileHead>>,
}

#[derive(Debug)]
pub(super) struct Remote<'tree> {
    hand: Partition,
    orders: Sender<Order<'tree>>,
    heads: Receiver<Vec<FileHead>>,
}

#[derive(Debug)]
pub(super) struct Linked<'tree> {
    reader: Reader<'tree>,
    remote: Remote<'tree>,
}

impl<'tree> Linked<'tree> {
    pub(super) fn new(packages: &'tree Packages, hand: Partition, replies: Sender<Reply<'tree>>) -> Linked<'tree> {
        let (order, orders) = mpsc::channel();
        let (head, heads) = mpsc::channel();
        Linked {
            reader: Reader { hand, packages, orders, replies, heads: head },
            remote: Remote { hand, orders: order, heads },
        }
    }

    pub(super) fn concluded<Out>(self, into: impl FnOnce(Reader<'tree>, Remote<'tree>) -> Out) -> Out {
        into(self.reader, self.remote)
    }
}

impl Reads for Reader<'_> {
    fn read(self, files: &impl Files) -> Readout {
        self.awaiting(files, BTreeMap::new())
    }
}

impl<'tree> Reader<'tree> {
    fn awaiting(self, files: &impl Files, kept: BTreeMap<Located<'tree>, SourceFile>) -> Readout {
        match self.orders.recv() {
            Ok(Order::Parse(nodes)) => self.parsed(files, kept, nodes),
            Ok(Order::Assemble(assignment)) => self.assembled(assignment, kept),
            Err(_) => Readout::default(),
        }
    }

    fn parsed(
        self,
        files: &impl Files,
        kept: BTreeMap<Located<'tree>, SourceFile>,
        nodes: Vec<Node<'tree>>,
    ) -> Readout {
        Parsed::read(files, self.packages, nodes, |report, fresh| {
            let mut held = kept;
            held.extend(fresh);
            self.replies.send(Reply::new(self.hand, report)).map_or_default(|()| self.awaiting(files, held))
        })
    }

    fn assembled(self, assignment: Assignment<'tree>, kept: BTreeMap<Located<'tree>, SourceFile>) -> Readout {
        let readout = assignment.readout(kept);
        let heads = readout.files().iter().map(|file| file.head().clone()).collect();
        self.heads.send(heads).map_or_default(|()| readout)
    }
}

impl<'tree> Remote<'tree> {
    pub(super) fn hand(&self) -> Partition {
        self.hand
    }

    pub(super) fn dispatched(&self, nodes: Vec<Node<'tree>>) -> Result<(), Vec<Node<'tree>>> {
        self.orders.send(Order::Parse(nodes)).map_err(|refused| refused.0.parsing())
    }

    pub(super) fn assigned(&self, assignment: Assignment<'tree>) -> Option<Assembling<'_>> {
        self.orders.send(Order::Assemble(assignment)).map_or(None, |()| Some(Assembling { heads: &self.heads }))
    }
}

#[derive(Debug)]
pub(super) struct Assembling<'remote> {
    heads: &'remote Receiver<Vec<FileHead>>,
}

impl Assembling<'_> {
    pub(super) fn heads(self) -> Vec<FileHead> {
        self.heads.recv().unwrap_or_default()
    }
}
