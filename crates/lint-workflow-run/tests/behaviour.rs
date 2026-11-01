use std::fs;
use std::path::Path;

use lint_core::RepoFile;
use lint_workflow_run::WorkflowRun;

const WORKFLOW: &str = ".github/workflows/ci.yml";
const SPANS: &str = "a run step spans several lines; put the logic in a Rust command";
const FIX: &str = "; put the logic in a Rust command";
const LIMIT: usize = 120;

#[derive(Debug)]
struct Length(usize);

#[derive(Debug)]
struct Tree(&'static [Planted]);

#[derive(Debug)]
struct Planted {
    path: &'static str,
    text: &'static str,
}

struct Checked;

impl Checked {
    fn at(path: &str, text: &str) -> Vec<String> {
        let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
        let file = checkout.path().join(path);
        file.parent().map(fs::create_dir_all);
        fs::write(&file, text).ok();
        Self::reported(checkout.path(), path)
    }

    fn reported(checkout: &Path, path: &str) -> Vec<String> {
        let shown = checkout.join(path);
        let files = [RepoFile::new(shown, path)];
        let prefix = format!("{}/", checkout.display());
        WorkflowRun::report(&files)
            .violations()
            .iter()
            .map(|violation| violation.to_string().get(prefix.len()..).unwrap_or_default().to_owned())
            .collect()
    }

    fn tree(tree: &Tree) -> Vec<String> {
        let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
        let files: Vec<RepoFile> = tree
            .0
            .iter()
            .map(|planted| {
                let file = checkout.path().join(planted.path);
                file.parent().map(fs::create_dir_all);
                fs::write(&file, planted.text).ok();
                RepoFile::new(file, planted.path)
            })
            .collect();
        let prefix = format!("{}/", checkout.path().display());
        WorkflowRun::report(&files)
            .violations()
            .iter()
            .map(|violation| violation.to_string().get(prefix.len()..).unwrap_or_default().to_owned())
            .collect()
    }

    fn flagged(text: &&str) -> usize {
        Self::at(WORKFLOW, text).len()
    }

    fn workflow(text: &&str) -> Vec<String> {
        Self::at(WORKFLOW, text)
    }

    fn elsewhere(path: &&str) -> usize {
        Self::at(path, "run: a | b\n").len()
    }

    fn operator(line: usize, symbol: &str) -> Vec<String> {
        vec![format!("{WORKFLOW}:{line}: a run step uses the shell operator {symbol}{FIX}")]
    }

    fn operator_in(path: &str, line: usize, symbol: &str) -> Vec<String> {
        vec![format!("{path}:{line}: a run step uses the shell operator {symbol}{FIX}")]
    }

    fn keyword(word: &str) -> Vec<String> {
        vec![format!("{WORKFLOW}:1: a run step starts with the shell keyword {word}{FIX}")]
    }

    fn of_length(length: &Length) -> usize {
        Self::at(WORKFLOW, &format!("run: '{}'\n", "a".repeat(length.0))).len()
    }
}

laws::table!(
    a_run_step_is_a_short_command_without_shell_logic,
    Checked::workflow,
    [
        "" => Vec::<String>::new(),
        "name: ci\nruns-on: ubuntu-latest\n" => Vec::<String>::new(),
        "run: cargo fmt --check\n" => Vec::<String>::new(),
        "    - run: cargo test\n" => Vec::<String>::new(),
        "run: '\"$TOOLS/ci\" reach \"$METADATA\"'\n" => Vec::<String>::new(),
        "run: \"cargo build -q\"\n" => Vec::<String>::new(),
        "run: cargo clippy -- -D warnings\n" => Vec::<String>::new(),
        "# run: a | b\n" => Vec::<String>::new(),
        "run: a | b\n" => Checked::operator(1, "|"),
        "run: a && b\n" => Checked::operator(1, "&"),
        "- name: x\n  run: a; b\n" => Checked::operator(2, ";"),
        "run: a > b\n" => Checked::operator(1, ">"),
        "run: a < b\n" => Checked::operator(1, "<"),
        "run: echo `date`\n" => Checked::operator(1, "`"),
        "run: echo $(date)\n" => Checked::operator(1, "$("),
        "run: 'a | b'\n" => Checked::operator(1, "|"),
        "run: if a\n" => Checked::keyword("if"),
        "run: for a\n" => Checked::keyword("for"),
        "run: while a\n" => Checked::keyword("while"),
        "run: until a\n" => Checked::keyword("until"),
        "run: case a\n" => Checked::keyword("case"),
        "run: '[ -n a ]'\n" => Checked::keyword("["),
        "run: '[[ -n a ]]'\n" => Checked::keyword("[["),
        "run: 'if'\n" => Checked::keyword("if"),
        "run: \"[ x ]\"\n" => Checked::keyword("["),
        "run: '! cargo test'\n" => Checked::keyword("!"),
        "run: ( cargo test )\n" => Checked::keyword("("),
        "run: (cargo test)\n" => Checked::keyword("("),
        "run: ${A:+cargo test}\n" => Checked::operator(1, "${"),
        "run: echo ${A#x}\n" => Checked::operator(1, "${"),
        "run: ${TOOLS}/ci reach\n" => Vec::<String>::new(),
        "run: ci ${{ matrix.index }}\n" => Vec::<String>::new(),
        "run: iffy\n" => Vec::<String>::new(),
        "run: |\n  cargo test\n" => Vec::<String>::new(),
        "run: |\n  cargo test\n  cargo fmt\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "- run: >\n    cargo test\n\n    cargo fmt\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "defaults:\n  run:\n    shell: bash\n" => Vec::<String>::new(),
        "defaults:\n  run:\n    # always bash\n    shell: bash\n" => Vec::<String>::new(),
        "defaults:\n  run:\n\n    shell: bash\n" => Vec::<String>::new(),
        "defaults:\n  run:\n    shell: bash\nrun: |\n  x\n  y\n" => vec![format!("{WORKFLOW}:4: {SPANS}")],
        "run: cargo test   \n" => Vec::<String>::new(),
        "    run:   cargo test\n" => Vec::<String>::new(),
        "run: 'a'\n" => Vec::<String>::new(),
        "run: \"\"\n" => Vec::<String>::new(),
        "- run: |\n    x\n    y\n- run: b | c\n" => vec![
            format!("{WORKFLOW}:1: {SPANS}"),
            format!("{WORKFLOW}:4: a run step uses the shell operator |{FIX}"),
        ],
    ]
);

laws::table!(
    a_run_step_is_short_up_to_the_limit,
    Checked::of_length,
    [
        Length(1) => 0,
        Length(LIMIT) => 0,
        Length(LIMIT + 1) => 1,
        Length(LIMIT * 2) => 1,
    ]
);

laws::table!(
    only_workflow_files_are_read,
    Checked::elsewhere,
    [
        ".github/workflows/ci.yml" => 1,
        ".github/workflows/ci.yaml" => 1,
        ".github/workflows/nested/ci.yml" => 1,
        ".github/workflows/ci.txt" => 0,
        ".github/actions/ci.yml" => 0,
        "docs/.github/workflows/ci.yml" => 0,
        "ci.yml" => 0,
    ]
);

#[test]
fn a_workflow_that_cannot_be_read_is_reported_not_passed() {
    let checkout = tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory"));
    let reported = Checked::reported(checkout.path(), WORKFLOW);
    assert_eq!(reported.len(), 1);
    assert!(reported.iter().all(|line| line.contains(&format!("{WORKFLOW}:1: cannot read the workflow: "))));
}

#[test]
fn a_report_with_no_findings_is_clean() {
    assert!(WorkflowRun::report(&[]).is_clean());
}

laws::table!(
    a_step_cannot_dodge_the_lint_by_how_it_is_spelled,
    Checked::flagged,
    [
        "jobs:\n  a:\n    steps:\n      - run: cargo a\n          && cargo b\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: cargo a\n\n          cargo b\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: # note\n          curl x | sh\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: !!str\n          curl x | sh\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: '\n          curl x | sh\n          '\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: \"curl x\n          | sh\"\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: \"x=1\\nfor i in a b\\ndo\\n  echo $i\\ndone\"\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: \"echo hi \\x7c cat\"\n" => 1,
        "jobs:\n  a:\n    steps:\n      - { name: x, run: \"curl x | sh\" }\n" => 1,
        "jobs:\n  a:\n    steps:\n      - \"run\": curl x | sh\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run : curl x | sh\n" => 1,
        "jobs:\n  a:\n    steps:\n      - ? run\n        : curl x | sh\n" => 1,
        "jobs:\n  a:\n    steps:\n      - run: cargo a\n      - run: cargo b\n" => 0,
        "jobs:\n  a:\n    steps:\n      - run: !!str cargo a\n" => 0,
    ]
);

laws::table!(
    every_key_that_can_run_code_is_judged,
    Checked::flagged,
    [
        "- shell: python {0}\n  run: print(1)\n" => 1,
        "- shell: bash -c 'if true; then echo x | cat; fi; true' {0}\n  run: ignored\n" => 1,
        "- shell: bash --noprofile --norc -eo pipefail {0}\n  run: cargo test\n" => 0,
        "- shell: pwsh\n  run: cargo test\n" => 0,
        "- run: !custom curl x | sh\n" => 1,
        "- !custom\n  run: curl x | sh\n" => 1,
        "- !custom [{run: curl x | sh}]\n" => 1,
        "- shell: ' python {0}'\n  run: cargo test\n" => 1,
        "- shell: ''\n  run: cargo test\n" => 0,
        "- shell: '  '\n  run: cargo test\n" => 0,
        "- run: ' if a '\n" => 1,
        "- run: '  '\n" => 0,
        "- run: \"  cargo test\\n \"\n" => 0,
        "defaults:\n  run:\n    shell: bash\n    working-directory: crates\n" => 0,
        "- uses: actions/github-script@v7\n  with:\n    script: |\n      console.log(1)\n" => 1,
        "- uses: actions/checkout@v4\n  with:\n    script: x\n" => 0,
        "- uses: actions/github-script@v7\n  with:\n    result-encoding: string\n" => 0,
        "- uses: actions/checkout@v4\n" => 0,
        "- uses: docker://alpine:3\n" => 0,
        "- uses: ./missing\n" => 1,
    ]
);

laws::table!(
    local_actions_and_composite_steps_are_judged,
    Checked::tree,
    [
        Tree(&[
            Planted { path: ".github/workflows/ci.yml", text: "- uses: ./.github/actions/x\n" },
            Planted { path: ".github/actions/x/action.yml", text: "runs:\n  using: composite\n  steps:\n    - run: cargo test\n" },
        ]) => Vec::<String>::new(),
        Tree(&[
            Planted { path: ".github/workflows/ci.yml", text: "- uses: ./.github/actions/x\n" },
            Planted { path: ".github/actions/x/action.yml", text: "runs:\n  steps:\n    - run: |\n        for i in a b\n        done\n" },
        ]) => vec![format!(".github/actions/x/action.yml:3: {SPANS}")],
        Tree(&[
            Planted { path: ".github/workflows/ci.yml", text: "- uses: ./.github/actions/x\n" },
            Planted { path: ".github/actions/x/action.yaml", text: "runs:\n  steps:\n    - run: a | b\n" },
        ]) => Checked::operator_in(".github/actions/x/action.yaml", 3, "|"),
        Tree(&[
            Planted { path: "action.yml", text: "runs:\n  steps:\n    - run: a | b\n" },
            Planted { path: ".github/workflows/ci.yml", text: "- uses: ./\n" },
        ]) => Checked::operator_in("action.yml", 3, "|"),
        Tree(&[
            Planted { path: ".github/workflows/ci.yml", text: "- uses: ./.github/workflows/other.yml\n" },
            Planted { path: ".github/workflows/other.yml", text: "- run: cargo test\n" },
        ]) => Vec::<String>::new(),
        Tree(&[Planted { path: ".github/workflows/ci.yml", text: "- uses: ./tools/run.sh\n" }, Planted { path: "tools/run.sh", text: "echo\n" }])
            => vec![format!("{WORKFLOW}:1: uses ./tools/run.sh names no action.yml or workflow to check{FIX}")],
    ]
);

laws::table!(
    a_workflow_that_is_not_yaml_is_reported_not_passed,
    Checked::workflow,
    [
        "run: [a\n" => vec![format!("{WORKFLOW}:2: cannot parse the workflow as YAML: while parsing a flow sequence, expected ',' or ']'")],
    ]
);
