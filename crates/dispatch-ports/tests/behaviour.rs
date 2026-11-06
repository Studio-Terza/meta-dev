use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::time::Duration;

use dispatch_ports::{
    AgentRun, Clock, ClockFiller, Deadline, Dispatch, DispatchFailure, DispatchFiller, Due, Ended, Finished, Pause,
    Pid, Prompt, Scratch, Spend, Started, Timeout, Tokens, Wall, clock_ops, dispatch_ops,
};
use laws::proptest::prelude::{prop_assert_eq, prop_oneof};
use laws::proptest::sample::select;
use laws::proptest::strategy::{Just, Strategy};
use port_core::{Calls, Pass, Released, Run};

struct Lists;

impl Lists {
    const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    fn started() -> impl Strategy<Value = String> {
        (select(&Self::WEEKDAYS[..]), select(&Self::MONTHS[..]), 1u8..=31, 0u8..24, 0u8..60, 0u8..60, 1970u16..3000)
            .prop_map(|(weekday, month, day, hour, minute, second, year)| {
                format!("{weekday} {month} {day:>2} {hour:02}:{minute:02}:{second:02} {year}    \n")
            })
    }

    fn result(subtype: &str, result: Option<&str>) -> String {
        let usage = serde_json::json!({
            "input_tokens": 11,
            "output_tokens": 7,
            "cache_creation_input_tokens": 3,
            "cache_read_input_tokens": 5
        });
        serde_json::json!({ "type": "result", "subtype": subtype, "is_error": false, "result": result, "usage": usage })
            .to_string()
    }
}

laws::roundtrip!(
    a_process_start_time_is_displayed_and_parsed_back,
    Lists::started().prop_filter_map("a start time", |text| text.parse::<Started>().ok()),
    |started: &Started| started.to_string(),
    |text: &String| text.parse::<Started>().ok(),
    Some
);

laws::table!(
    text_that_is_no_start_time_is_refused,
    |text: &&str| text.parse::<Started>().is_ok(),
    [
        "" => false,
        "Fri Nov  6 01:16:00" => false,
        "Fri Nov  6 01:16 2026" => false,
        "Fry Nov  6 01:16:00 2026" => false,
        "Fri Nov  6 01:16:00 2026 extra" => false,
        "ps: process id too large: 999999" => false,
        "Fri Nov  6 01:16:00 2026" => true,
    ]
);

laws::law!(
    a_deadline_is_due_from_its_spawn_plus_its_timeout_onward,
    (0i64..1_000_000, 0u32..100_000, -200_000i64..200_000),
    case,
    {
        let (spawn, timeout, offset) = case;
        let deadline = Deadline::counted_from(Wall::new(spawn), Timeout::new(timeout));
        let now = spawn + i64::from(timeout) + offset;
        let expected = if offset >= 0 { Due::Passed } else { Due::Pending };
        prop_assert_eq!(deadline.due(Wall::new(now)), expected);
    }
);

laws::law!(a_wall_after_a_pause_is_later_by_that_many_seconds, (0i64..1_000_000, 0u32..100_000), case, {
    let (start, pause) = case;
    prop_assert_eq!(Wall::new(start).after(Pause::new(pause)), Wall::new(start + i64::from(pause)));
    prop_assert_eq!(Duration::from(Pause::new(pause)), Duration::from_secs(u64::from(pause)));
});

laws::law!(a_wall_is_read_from_the_seconds_since_the_epoch, 0u64..4_000_000_000, seconds, {
    let wall = Wall::from(Duration::from_secs(seconds) + Duration::from_millis(999));
    prop_assert_eq!(Some(wall), i64::try_from(seconds).ok().map(Wall::new));
});

#[test]
fn a_successful_result_yields_its_reply_and_every_spend_of_its_usage() {
    let finished = Lists::result("success", Some("conversational(85b287b3.0)")).parse::<Finished>().ok();
    let reply = finished.as_ref().map(|finished| finished.reply().to_string());
    assert_eq!(reply.as_deref(), Some("conversational(85b287b3.0)"));
    let usage = finished.map(|finished| finished.usage());
    let spent: Vec<Option<Tokens>> = [Spend::Input, Spend::Output, Spend::CacheWritten, Spend::CacheRead]
        .iter()
        .map(|spend| usage.map(|usage| spend.spent_in(&usage)))
        .collect();
    assert_eq!(spent, [11, 7, 3, 5].map(|count| Some(Tokens::new(count))));
    assert_eq!(
        usage.map(|usage| usage.to_string()).as_deref(),
        Some("usage input 11 output 7 cache-written 3 cache-read 5")
    );
}

laws::table!(
    only_a_successful_result_with_a_reply_is_finished,
    |text: &&str| text.parse::<Finished>().is_ok(),
    [
        Lists::result("success", Some("reply")).as_str() => true,
        Lists::result("error_max_turns", Some("reply")).as_str() => false,
        Lists::result("success", None).as_str() => false,
        "not json" => false,
    ]
);

#[test]
fn a_kill_that_signals_reports_killed_and_one_that_finds_nothing_reports_absent() {
    let killed: Ended = ExitStatus::from_raw(0).into();
    let absent: Ended = ExitStatus::from_raw(256).into();
    assert_eq!([killed, absent], [Ended::Killed, Ended::Absent]);
    assert_eq!([killed.to_string(), absent.to_string()], ["killed", "absent"]);
}

#[derive(Debug, Default)]
struct Silent;

impl DispatchFiller for Silent {
    fn dispatch(
        &self,
        _pass: Pass<'_, dispatch_ops::Dispatch>,
        _prompt: &Prompt,
        deadline: Deadline,
        _scratch: &Scratch,
    ) -> Result<AgentRun, DispatchFailure> {
        Ok(AgentRun::new(Pid::new(7), deadline))
    }

    fn output(
        &self,
        _pass: Pass<'_, dispatch_ops::Output>,
        _run: &mut AgentRun,
    ) -> Result<Option<Finished>, DispatchFailure> {
        Ok(None)
    }

    fn started(&self, _pass: Pass<'_, dispatch_ops::Started>, _pid: Pid) -> Result<Option<Started>, DispatchFailure> {
        Ok(None)
    }

    fn kill(&self, _pass: Pass<'_, dispatch_ops::Kill>, _pid: Pid) -> Result<Ended, DispatchFailure> {
        Ok(Ended::Absent)
    }
}

#[derive(Debug)]
struct Frozen(Wall);

impl ClockFiller for Frozen {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        self.0
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause) -> Wall {
        self.0.after(pause)
    }
}

#[test]
fn a_dispatched_run_is_held_with_its_deadline_and_released_once() -> Result<(), DispatchFailure> {
    let run = Run::root();
    let clock = run.port(Frozen(Wall::new(100)));
    let dispatch = run.port(Silent);
    let deadline = Deadline::counted_from(clock.now(), Timeout::new(5));
    let mut held = dispatch.dispatch(&Prompt::from("p"), deadline, &Scratch::from(std::path::PathBuf::from("/s")))?;
    let polled = dispatch.output(&mut held)?;
    assert_eq!((held.pid(), held.deadline(), polled), (Pid::new(7), Deadline::new(Wall::new(105)), None));
    assert_eq!(clock.waited(Pause::new(2)), Wall::new(102));
    held.end().ok();
    let snapshot = run.finish().snapshot().clone();
    assert_eq!(snapshot.calls::<dispatch_ops::Dispatch>(), Calls::new(1));
    assert_eq!(snapshot.calls::<Released<AgentRun>>(), Calls::new(1));
    Ok(())
}

laws::law!(any_prompt_text_is_kept_whole, prop_oneof![Just(String::new()), "[^\u{0}]{0,40}"], text, {
    let prompt = Prompt::from(text.as_str());
    prop_assert_eq!(prompt.as_ref(), text.as_str());
});
