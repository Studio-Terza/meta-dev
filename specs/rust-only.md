# rust-only

Problem: every lint stops at the language boundary, so logic in other languages escapes them.

L1 Durable logic in a repository is Rust. Web assets live only under a directory named `assets`.
L2 No allow-lists.
D1 Lint `rust_only`: source files in other languages fail by extension, outside `assets`.
D2 Lint on workflows: a `run:` step is at most a short command; logic does not live in YAML. after: D1
D3 Port this repository's .sh and .py files, and CI logic in workflows, to Rust. after: D2
Out: throwaway commands, Dockerfiles, configuration files.
