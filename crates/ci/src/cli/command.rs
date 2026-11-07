mod failure;
mod step;

use std::ffi::OsString;
use std::io::Write;

use step::cold_size::ColdSize;
use step::github::{CompileTrend, Labels, Show, SpecLine, StageIssues, Summarize, WallTimes};
use step::mutation::{Matrix, MutantsChanged, MutantsShard, ShardSummary};
use step::ratchet::{MeasuredHistory, RatchetLower};
use step::{CompileTimes, History, InstallTools, LibtestSeconds, Logged, ReachScope, Step, Timed, ToolsKey};

use super::ports::{Exit, Ports};
pub(crate) use failure::Failure;

#[derive(Debug)]
pub enum Command {
    Timed(Timed),
    LibtestSeconds(LibtestSeconds),
    Reach(ReachScope),
    History(History),
    MeasuredHistory(MeasuredHistory),
    RatchetLower(RatchetLower),
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
    SpecLine(SpecLine),
    StageIssues(StageIssues),
    CompileTrend(CompileTrend),
    ShardSummary(ShardSummary),
    Matrix(Matrix),
    ColdSize(ColdSize),
}

impl Command {
    const TIMED: &'static str = "timed";
    const LIBTEST_SECONDS: &'static str = "libtest-seconds";
    const REACH: &'static str = "reach";
    const HISTORY: &'static str = "history";
    const MEASURED_HISTORY: &'static str = "measured-history";
    const RATCHET_LOWER: &'static str = "ratchet-lower";
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
    const SPEC_LINE: &'static str = "spec-line";
    const STAGE_ISSUES: &'static str = "stage-issues";
    const COMPILE_TREND: &'static str = "compile-trend";
    const SHARD_SUMMARY: &'static str = "shard-summary";
    const MATRIX: &'static str = "matrix";
    const COLD_SIZE: &'static str = "cold-size";

    pub fn parse(mut arguments: impl Iterator<Item = OsString>) -> Result<Self, Failure<'static>> {
        match arguments.next().and_then(|name| name.into_string().ok()).as_deref() {
            Some(Self::TIMED) => Self::parsed(arguments.collect(), Command::Timed),
            Some(Self::LIBTEST_SECONDS) => Self::parsed(arguments.collect(), Command::LibtestSeconds),
            Some(Self::REACH) => Self::parsed(arguments.collect(), Command::Reach),
            Some(Self::HISTORY) => Self::parsed(arguments.collect(), Command::History),
            Some(Self::MEASURED_HISTORY) => Self::parsed(arguments.collect(), Command::MeasuredHistory),
            Some(Self::RATCHET_LOWER) => Self::parsed(arguments.collect(), Command::RatchetLower),
            Some(Self::COMPILE_TIMES) => Self::parsed(arguments.collect(), Command::CompileTimes),
            Some(Self::LOGGED) => Self::parsed(arguments.collect(), Command::Logged),
            Some(Self::TOOLS) => Self::parsed(arguments.collect(), Command::Tools),
            Some(Self::TOOLS_KEY) => Self::parsed(arguments.collect(), Command::ToolsKey),
            Some(Self::MUTANTS_SHARD) => Self::parsed(arguments.collect(), Command::MutantsShard),
            Some(Self::MUTANTS_CHANGED) => Self::parsed(arguments.collect(), Command::MutantsChanged),
            Some(Self::WALL_TIMES) => Self::parsed(arguments.collect(), Command::WallTimes),
            Some(Self::SHOW) => Self::parsed(arguments.collect(), Command::Show),
            Some(Self::SUMMARIZE) => Self::parsed(arguments.collect(), Command::Summarize),
            Some(Self::LABELS) => Self::parsed(arguments.collect(), Command::Labels),
            Some(Self::SPEC_LINE) => Self::parsed(arguments.collect(), Command::SpecLine),
            Some(Self::STAGE_ISSUES) => Self::parsed(arguments.collect(), Command::StageIssues),
            Some(Self::COMPILE_TREND) => Self::parsed(arguments.collect(), Command::CompileTrend),
            Some(Self::SHARD_SUMMARY) => Self::parsed(arguments.collect(), Command::ShardSummary),
            Some(Self::MATRIX) => Self::parsed(arguments.collect(), Command::Matrix),
            Some(Self::COLD_SIZE) => Self::parsed(arguments.collect(), Command::ColdSize),
            _ => Err(Failure::Usage),
        }
    }

    fn parsed<S: Step>(arguments: Vec<OsString>, variant: fn(S) -> Self) -> Result<Self, Failure<'static>> {
        S::from_arguments(arguments).map(variant)
    }

    pub fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        match self {
            Self::Timed(step) => step.carry_out(ports, out),
            Self::LibtestSeconds(step) => step.carry_out(ports, out),
            Self::Reach(step) => step.carry_out(ports, out),
            Self::History(step) => step.carry_out(ports, out),
            Self::MeasuredHistory(step) => step.carry_out(ports, out),
            Self::RatchetLower(step) => step.carry_out(ports, out),
            Self::CompileTimes(step) => step.carry_out(ports, out),
            Self::Logged(step) => step.carry_out(ports, out),
            Self::Tools(step) => step.carry_out(ports, out),
            Self::ToolsKey(step) => step.carry_out(ports, out),
            Self::MutantsShard(step) => step.carry_out(ports, out),
            Self::MutantsChanged(step) => step.carry_out(ports, out),
            Self::WallTimes(step) => step.carry_out(ports, out),
            Self::Show(step) => step.carry_out(ports, out),
            Self::Summarize(step) => step.carry_out(ports, out),
            Self::Labels(step) => step.carry_out(ports, out),
            Self::SpecLine(step) => step.carry_out(ports, out),
            Self::StageIssues(step) => step.carry_out(ports, out),
            Self::CompileTrend(step) => step.carry_out(ports, out),
            Self::ShardSummary(step) => step.carry_out(ports, out),
            Self::Matrix(step) => step.carry_out(ports, out),
            Self::ColdSize(step) => step.carry_out(ports, out),
        }
    }
}
