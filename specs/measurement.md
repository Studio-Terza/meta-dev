# measurement

Problem: slow code and creep must show up early, without enumerating every cause.

L1 Wall time of the test suite and CI steps is recorded on every main run.
L2 Blocking limits are set by the owner, never chosen by an agent.
L3 Counts are deterministic: the same on any machine.
D1 CI prints a report on main: wall time per step, tests, test lines and process starts per crate.
D2 Lint: process starts are counted through the `ports` convention and reported per crate. after: D1
D3 Wall-time limits with warn and block levels. after: D1
D4 Local token report: spend per day, model, lead and agents, from transcripts. Kept outside this repository.
D5 One tracking table per project: details done, spend, attached share, features delivered. after: D4
D6 Scheduled cold build on main: daily and on manual dispatch, with no build cache and no sccache, the clippy and test-compile stages each build from scratch in their own target directory, timed per stage and per crate from `--timings`. The stage totals are appended to the history file main keeps. PR and main-push stages are judged as before. after: D1
D7 Cold build ratchet: `ci/cold-build.toml` records one value per stage, in the measured file's format. A run above the record, or a stage without one, fails and opens an issue for a fix-forward on main. `ci ratchet-lower <record> <measured>` writes a run's values when none is above the record; it never raises one, and it refuses a warn and block limit. The first record is the highest value of the first runs. after: D6
Out: runner noise, per-PR judging.

## History

D6, D7: owner's messages 236ed180 and 66840525. The ratchet follows the local tools lint ratchet: a recorded value in a data file, a check that fails above it, and a lowering command. The record is measured, not chosen.
