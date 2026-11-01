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

    fn workflow(text: &&str) -> Vec<String> {
        Self::at(WORKFLOW, text)
    }

    fn elsewhere(path: &&str) -> usize {
        Self::at(path, "run: |\n  echo hi\n").len()
    }

    fn operator(line: usize, symbol: &str) -> Vec<String> {
        vec![format!("{WORKFLOW}:{line}: a run step uses the shell operator {symbol}{FIX}")]
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
        "name: x\n- run: a; b\n" => Checked::operator(2, ";"),
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
        "run: [ -n a ]\n" => Checked::keyword("["),
        "run: [[ -n a ]]\n" => Checked::keyword("[["),
        "run: 'if'\n" => Checked::keyword("if"),
        "run: \"[ x ]\"\n" => Checked::keyword("["),
        "run: iffy\n" => Vec::<String>::new(),
        "run: |\n  cargo test\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "run: |-\n  cargo test\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "- run: >\n    cargo test\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "run:\n  cargo test\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "run:\n" => vec![format!("{WORKFLOW}:1: {SPANS}")],
        "defaults:\n  run:\n    shell: bash\n" => Vec::<String>::new(),
        "defaults:\n  run:\n    shell: bash\nrun: |\n  x\n" => vec![format!("{WORKFLOW}:4: {SPANS}")],
        "run: cargo test   \n" => Vec::<String>::new(),
        "    run:   cargo test\n" => Vec::<String>::new(),
        "run: 'a'\n" => Vec::<String>::new(),
        "run: '\n" => Vec::<String>::new(),
        "run: \"\"\n" => Vec::<String>::new(),
        "a: 1\nrun: |\n  x\nrun: b | c\n" => vec![
            format!("{WORKFLOW}:2: {SPANS}"),
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
