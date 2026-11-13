use code_model::Model;
use code_reader::Reader;
use law_facts::Laws;
use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use lint_transition_graph::{Extraction, GraphLint, Transitions, TypeSystem};

pub(crate) struct OnGraph<Graph>(PhantomData<Graph>);

impl<Graph: GraphLint> Lint for OnGraph<Graph> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let extraction = Extraction::from(files);
        let laws = Laws::from(files);
        let model: &Model = Box::leak(Box::new(Reader::read_files(files)));
        let transitions = Transitions::from(model);
        Graph::report_graph(&TypeSystem::built_from(&extraction, &laws, &transitions))
    }
}
