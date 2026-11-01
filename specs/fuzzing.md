# fuzzing

Problem: parsers and lints meet inputs nobody wrote a test for.

L1 Fuzzing is a separate job and blocks no PR.
L2 A crash found becomes a named unit test, and its input is kept.
D1 Fuzz targets for the lints' source handling: arbitrary bytes and arbitrary valid syntax into the runner must not panic.
D2 Scheduled job on main with a fixed time limit, reporting crashes as a failed run.
Out: coverage counts, fuzzing in PR CI, counting toward mutation.
