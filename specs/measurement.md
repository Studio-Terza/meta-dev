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
D7 Cold build measures only: each run records its stage times (cold-clippy, cold-test-compile) in main's history and the job summary, with per-crate times from `--timings`. It never fails on time, opens no issue other than the trend issue of D9, and never changes a record file. `ci/cold-build.toml`, `ci ratchet-lower` and `ci stage-issues` stay in the repository, unused by it. after: D6
D8 The test-compile limit is judged and printed on every pull request and push but never fails the job. Every other stage limit blocks on pull requests into `project/*` branches exactly as on pull requests into main, and those pull requests build and judge the whole workspace as pull requests into main do, never only the crates the change reaches. Pushes to `project/*` branches run CI and build the same caches main saves, judged as main pushes are, so pull requests into a project branch restore their base branch's cache. after: D3
D9 Compile-time trend: the cold build compares its total wall time (the sum of its stage times) with each successful cold build on main in the previous 14 days. When today's total is slower than every one of them, it opens one issue titled as a compile-time trend, or updates the open one, whose body gives the total against the fortnight's range, the crates whose times grew most against their fortnight average (from `--timings`), and the pull requests merged into main in the fortnight, as context. The threshold is derived from the history, not chosen. after: D7, D8
Out: runner noise, per-PR judging.

## History

D6, D7: owner's messages 236ed180 and 66840525. The ratchet follows the local tools lint ratchet: a recorded value in a data file, a check that fails above it, and a lowering command. The record is measured, not chosen.

D7: owner's message cd1ac5fe replaced the ratchet with measuring only; the data decides later what to do with it (earlier form: 66840525).

D8: owner's message f45c05cd, after a cold-cache test-compile of 55.9s against the 18s block on main. Project-branch pushes building caches: owner f45c05cd.

D8: whole-workspace reach makes test-compile report-only: owner a3b8e0a3.

D8: pull requests into project branches block on clippy, test-run and lint as pull requests into main do, reversing PR #240's report-only exemption, and are judged on the whole workspace so slicing a change cannot fit it under a limit: owner 54283ee7.

D8, D9: owner's messages 36da5bd4, a3b8e0a3 and f45c05cd: test-compile never blocks; a trend message on total wall time replaces the per-pull-request block (earlier forms: blocking on `project/*` pull requests, then report-only on whole-workspace reach).
