# rust-only

Problem: every lint stops at the language boundary, so logic in other languages escapes them.

L1 Durable logic in a repository is Rust. Web assets live only under a directory named `assets`; code a platform requires in its own language, such as XCUITest Swift, lives only under a directory named `platform`, and only as test code.
L2 No allow-lists.
L3 A renamed or disguised source file is source.
D1 Lint `rust_only`: source files in other languages fail, outside `assets` and outside test code under `platform`. Source is found by extension, by a double extension hiding a source extension (`.swift.txt`, `.py.bak`), and by content: a shebang for a non-Rust interpreter, or lines that clearly are Python, Swift, shell or C. Ordinary text and data files pass.
D2 Lint `workflow_run` on workflows, read as YAML, and on local composite actions: the value of a `run` key is one line of at most 120 characters with no shell operator or control keyword, a `shell` key names a shell, and a script step holds no `script`; logic does not live in YAML. after: D1
D3 Port this repository's .sh and .py files, and CI logic in workflows, to Rust: every multi-line or piped `run:` step is a `ci` command. after: D2
Out: throwaway commands, Dockerfiles, configuration files.
