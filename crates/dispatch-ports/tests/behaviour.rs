use dispatch_ports::{Finished, Reply, Tokens, Usage};
use newtype::Newtype;

struct Envelope;

impl Envelope {
    fn read(text: &str) -> Option<(String, [u64; 4])> {
        text.parse::<Finished>()
            .ok()
            .map(|finished| (finished.reply().clone().into_inner(), finished.usage().spent().map(Tokens::into_inner)))
    }
}

laws::table!(
    an_agent_result_is_read_as_its_reply_and_usage,
    |text: &&'static str| Envelope::read(text),
    [
        r#"{"type":"result","subtype":"success","is_error":false,"result":"conversational(a.0)","usage":{"input_tokens":4,"output_tokens":9,"cache_creation_input_tokens":2,"cache_read_input_tokens":7,"service_tier":"standard"}}"# => Some(("conversational(a.0)".to_owned(), [4, 9, 2, 7])),
        r#"{"subtype":"success","result":"x","usage":{"input_tokens":1,"output_tokens":2}}"# => Some(("x".to_owned(), [1, 2, 0, 0])),
        r#"{"subtype":"error_max_turns","result":"x","usage":{"input_tokens":1,"output_tokens":2}}"# => None,
        r#"{"subtype":"success","usage":{"input_tokens":1,"output_tokens":2}}"# => None,
        r#"{"subtype":"success","result":"x"}"# => None,
        "not json" => None,
    ]
);

#[test]
fn a_finished_run_holds_what_it_was_given() {
    let usage = Usage::new(Tokens::new(1), Tokens::new(2), Tokens::new(3), Tokens::new(4));
    let finished = Finished::new(Reply::new("r".to_owned()), usage);
    assert_eq!((finished.reply().clone(), finished.usage()), (Reply::new("r".to_owned()), usage));
}
