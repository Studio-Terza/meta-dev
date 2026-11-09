#![deny(clippy::wildcard_enum_match_arm)]

use grammar_timing_page::Build;

laws::table!(
    a_timings_page_records_a_failed_build_only_in_its_error_cell,
    Build::recorded,
    [
        "<html><table><tr><td>Total time:</td><td>0.5s</td></tr></table></html>" => Build::Finished,
        "<tr><td class=\"error-text\">Error:</td><td>1 job failed</td></tr>" => Build::Failed,
        "<td class=\"error-text\">" => Build::Failed,
        "error-text Error: 1 job failed" => Build::Finished,
        "<td class=\"error\">Error:</td>" => Build::Finished,
        "" => Build::Finished,
    ]
);
