mod failure;
mod step;

use std::ffi::OsString;
use std::io::Write;

use step::github::{Labels, Show, StageIssues, Summarize, WallTimes};
use step::mutation::{Matrix, MutantsChanged, MutantsShard, ShardSummary};
use step::{CompileTimes, History, InstallTools, LibtestSeconds, Logged, ReachScope, Step, Timed, ToolsKey};

use super::ports::{Exit, Ports};
use failure::Failure;

#[derive(Debug)]
pub enum Command {
    Timed(Timed),
    LibtestSeconds(LibtestSeconds),
    Reach(ReachScope),
    History(History),
    CompileTimes(CompileTimes),
    Logged(Logged),
    Tools(InstallTools),
    ToolsKey(ToolsKey),
    MutantsShard(MutantsShard),
    MutantsChanged(MutantsChanged),
    WallTimes(WallTimes),
    Show(Show),
    Summarize(Summarize),
    Labels(Labels),
    StageIssues(StageIssues),
    ShardSummary(ShardSummary),
    Matrix(Matrix),
}

impl Command {
    const TIMED: &'static str = "timed";
    const LIBTEST_SECONDS: &'static str = "libtest-seconds";
    const REACH: &'static str = "reach";
    const HISTORY: &'static str = "history";
    const COMPILE_TIMES: &'static str = "compile-times";
    const LOGGED: &'static str = "logged";
    const TOOLS: &'static str = "tools";
    const TOOLS_KEY: &'static str = "tools-key";
    const MUTANTS_SHARD: &'static str = "mutants-shard";
    const MUTANTS_CHANGED: &'static str = "mutants-changed";
    const WALL_TIMES: &'static str = "wall-times";
    const SHOW: &'static str = "show";
    const SUMMARIZE: &'static str = "summarize";
    const LABELS: &'static str = "labels";
    const STAGE_ISSUES: &'static str = "stage-issues";
    const SHARD_SUMMARY: &'static str = "shard-summary";
    const MATRIX: &'static str = "matrix";

    pub fn parse(mut arguments: impl Iterator<Item = OsString>) -> Result<Command, Failure> {
        let name = arguments.next().and_then(|name| name.into_string().ok());
        let rest: Vec<OsString> = arguments.collect();
        match name.as_deref() {
            Some(Self::TIMED) => Timed::from_arguments(&rest).map(Command::Timed),
            Some(Self::LIBTEST_SECONDS) => LibtestSeconds::from_arguments(&rest).map(Command::LibtestSeconds),
            Some(Self::REACH) => ReachScope::from_arguments(&rest).map(Command::Reach),
            Some(Self::HISTORY) => History::from_arguments(&rest).map(Command::History),
            Some(Self::COMPILE_TIMES) => CompileTimes::from_arguments(&rest).map(Command::CompileTimes),
            Some(Self::LOGGED) => Logged::from_arguments(&rest).map(Command::Logged),
            Some(Self::TOOLS) => InstallTools::from_arguments(&rest).map(Command::Tools),
            Some(Self::TOOLS_KEY) => ToolsKey::from_arguments(&rest).map(Command::ToolsKey),
            Some(Self::MUTANTS_SHARD) => MutantsShard::from_arguments(&rest).map(Command::MutantsShard),
            Some(Self::MUTANTS_CHANGED) => MutantsChanged::from_arguments(&rest).map(Command::MutantsChanged),
            Some(Self::WALL_TIMES) => WallTimes::from_arguments(&rest).map(Command::WallTimes),
            Some(Self::SHOW) => Show::from_arguments(&rest).map(Command::Show),
            Some(Self::SUMMARIZE) => Summarize::from_arguments(&rest).map(Command::Summarize),
            Some(Self::LABELS) => Labels::from_arguments(&rest).map(Command::Labels),
            Some(Self::STAGE_ISSUES) => StageIssues::from_arguments(&rest).map(Command::StageIssues),
            Some(Self::SHARD_SUMMARY) => ShardSummary::from_arguments(&rest).map(Command::ShardSummary),
            Some(Self::MATRIX) => Matrix::from_arguments(&rest).map(Command::Matrix),
            _ => Err(Failure::Usage),
        }
    }

    pub fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        match self {
            Command::Timed(step) => step.carry_out(ports, out),
            Command::LibtestSeconds(step) => step.carry_out(ports, out),
            Command::Reach(step) => step.carry_out(ports, out),
            Command::History(step) => step.carry_out(ports, out),
            Command::CompileTimes(step) => step.carry_out(ports, out),
            Command::Logged(step) => step.carry_out(ports, out),
            Command::Tools(step) => step.carry_out(ports, out),
            Command::ToolsKey(step) => step.carry_out(ports, out),
            Command::MutantsShard(step) => step.carry_out(ports, out),
            Command::MutantsChanged(step) => step.carry_out(ports, out),
            Command::WallTimes(step) => step.carry_out(ports, out),
            Command::Show(step) => step.carry_out(ports, out),
            Command::Summarize(step) => step.carry_out(ports, out),
            Command::Labels(step) => step.carry_out(ports, out),
            Command::StageIssues(step) => step.carry_out(ports, out),
            Command::ShardSummary(step) => step.carry_out(ports, out),
            Command::Matrix(step) => step.carry_out(ports, out),
        }
    }
}
