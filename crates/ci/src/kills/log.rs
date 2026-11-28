use grammar_lines::Lines;
use grammar_test_log::{TestLine, Verdict};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct Test {
    package: String,
    binary: String,
    name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Golden,
    Other,
}

#[derive(Debug, Default)]
pub(crate) struct Log {
    runs: Vec<Run>,
}

#[derive(Debug)]
struct Run {
    test: Test,
    verdict: Verdict,
}

#[derive(Debug)]
struct Binary {
    package: String,
    binary: String,
}

#[derive(Debug, Default)]
struct Reading {
    binary: Option<Binary>,
    runs: Vec<Run>,
}

impl Test {
    const GOLDEN: &'static str = "golden";

    pub(crate) fn kind(&self) -> Kind {
        if self.name.contains(Test::GOLDEN) { Kind::Golden } else { Kind::Other }
    }
}

impl Log {
    pub(crate) fn read(text: &str) -> Log {
        let reading =
            text.parse::<Lines>().into_iter().flatten().fold(Reading::default(), |reading, line| reading.line(&line));
        Log { runs: reading.runs }
    }

    pub(crate) fn tests(&self) -> impl Iterator<Item = &Test> {
        self.runs.iter().map(|run| &run.test)
    }

    pub(crate) fn failed(&self) -> impl Iterator<Item = &Test> {
        self.runs.iter().filter(|run| run.verdict == Verdict::Failed).map(|run| &run.test)
    }
}

impl Reading {
    fn line(mut self, line: &str) -> Reading {
        match TestLine::read(line) {
            Some(TestLine::Running { package, binary }) => {
                Reading { binary: Some(Binary { package: package.to_owned(), binary: binary.to_owned() }), ..self }
            }
            Some(TestLine::Ran { name, verdict }) => {
                let run = self.binary.as_ref().map(|binary| Run { test: binary.test(name), verdict });
                self.runs.extend(run);
                self
            }
            None => self,
        }
    }
}

impl Binary {
    fn test(&self, name: &str) -> Test {
        Test { package: self.package.clone(), binary: self.binary.clone(), name: name.to_owned() }
    }
}
