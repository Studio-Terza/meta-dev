use code_reader::Reader;
use law_facts::Laws;
use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use lint_transition_graph::{Extraction, GraphLint, TypeSystem};

pub(crate) struct OnGraph<Graph>(PhantomData<Graph>);

impl<Graph: GraphLint> Lint for OnGraph<Graph> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let extraction = Extraction::from(files);
        let laws = Laws::from(files);
        let model = Reader::read_files(files);
        Graph::report_graph(&TypeSystem::built_from(&extraction, &laws, &model))
    }
}
