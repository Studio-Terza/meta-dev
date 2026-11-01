# rust-only

Problem: every lint stops at the language boundary, so logic in other languages escapes them.

L1 Durable logic in a repository is Rust. Web assets live only under a directory named `assets`.
L2 No allow-lists.
D1 Lint `rust_only`: source files in other languages fail by extension, outside `assets`; a file with no extension fails when its first line is a shebang for a non-Rust interpreter.
D2 Lint `workflow_run` on workflows, read as YAML, and on local composite actions: the value of a `run` key is one line of at most 120 characters with no shell operator or control keyword, a `shell` key names a shell, and a script step holds no `script`; logic does not live in YAML. after: D1
D3 Port this repository's .sh and .py files, and CI logic in workflows, to Rust: every multi-line or piped `run:` step is a `ci` command. after: D2
Out: throwaway commands, Dockerfiles, configuration files.
