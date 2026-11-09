use crate::provider::{
    Change, Options, Prompt, Request, Role, Scenario, UsageKind,
    protocol::{self, Decoder, MAX_BYTES},
};
use serde_json::{Value, json};

pub(super) fn request(streaming: bool) -> Request {
    Request {
        id: 7,
        messages: vec![Prompt {
            role: Role::User,
            text: "Hi".into(),
        }],
        scenario: Scenario::Normal,
        options: Options {
            model: "gpt-6-luna".into(),
            effort: "low".into(),
            system_prompt: "Be concise.".into(),
            streaming,
        },
    }
}
pub(super) fn response() -> Value {
    json!({"id":"resp_fixture","model":"gpt-6-luna","status":"completed",
        "output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Hi! How can I help?"}]}],
        "usage":{"input_tokens":12,"output_tokens":8}})
}
fn event(value: Value) -> String {
    format!("data: {value}\n\n")
}

#[test]
fn request_uses_responses_schema_and_does_not_serialize_credentials() {
    let mut r = request(true);
    r.messages.push(Prompt {
        role: Role::Assistant,
        text: "Earlier reply".into(),
    });
    let body: Value = serde_json::from_slice(&protocol::body(&r).unwrap()).unwrap();
    assert_eq!(body["model"], "gpt-6-luna");
    assert_eq!(body["reasoning"]["effort"], "low");
    assert_eq!(body["instructions"], "Be concise.");
    assert_eq!(body["store"], false);
    assert_eq!(body["stream"], true);
    assert_eq!(body["input"][0]["content"], "Hi");
    assert_eq!(body["input"][1]["role"], "assistant");
    assert!(body.get("api_key").is_none());
    assert!(body.get("Authorization").is_none());
    r.options.model.clear();
    assert!(protocol::body(&r).is_err());
    r.options.model = "gpt-6-luna".into();
    r.messages[0].text = "x".repeat(MAX_BYTES + 1);
    assert!(protocol::body(&r).is_err());
}

#[test]
fn fragmented_utf8_crlf_and_multiline_sse_emit_one_completion() {
    let stream = format!(
        ": keepalive\r\nevent: response.output_text.delta\r\ndata: {{\"type\":\"response.output_text.delta\",\r\ndata: \"delta\":\"Hé 🦀\"}}\r\n\r\n{}{}",
        event(json!({"type":"response.output_text.done","text":"Hé 🦀"})),
        event(json!({"type":"response.completed","response":response()}))
    );
    let mut decoder = Decoder::new(true);
    let mut changes = Vec::new();
    for byte in stream.as_bytes() {
        changes.extend(decoder.feed(&[*byte]));
    }
    changes.extend(decoder.finish());
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0], Change::Delta("Hé 🦀".into()));
    let Change::Complete(usage) = &changes[1] else {
        panic!("completion missing")
    };
    assert_eq!(usage.kind, UsageKind::Reported);
    assert_eq!(usage.input, 12);
    assert_eq!(usage.output, 8);
    assert_eq!(usage.response_id.as_deref(), Some("resp_fixture"));
    assert!(decoder.feed(b"data: invalid\n\n").is_empty());
}

#[test]
fn buffered_and_completion_only_payloads_recover_text_without_duplication() {
    for streaming in [false, true] {
        let mut decoder = Decoder::new(streaming);
        let raw = if streaming {
            event(json!({"type":"response.completed","response":response()})).into_bytes()
        } else {
            serde_json::to_vec(&response()).unwrap()
        };
        let mut changes = Vec::new();
        for chunk in raw.chunks(3) {
            changes.extend(decoder.feed(chunk));
        }
        changes.extend(decoder.finish());
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0], Change::Delta("Hi! How can I help?".into()));
        assert!(matches!(changes[1], Change::Complete(_)));
    }
}

#[test]
fn truncation_malformed_events_and_size_limits_are_failures() {
    for bytes in [
        b"data: invalid\n\n".as_slice(),
        b"data: [DONE]\n\n",
        b"data: {\"type\":\"response.incomplete\"}\n\n",
    ] {
        let mut decoder = Decoder::new(true);
        assert!(
            decoder
                .feed(bytes)
                .iter()
                .any(|c| matches!(c, Change::Failed(_)))
        );
        assert!(decoder.finish().is_empty());
    }
    let mut decoder = Decoder::new(true);
    assert_eq!(
        decoder
            .feed(event(json!({"type":"response.output_text.delta","delta":"partial"})).as_bytes()),
        vec![Change::Delta("partial".into())]
    );
    assert!(matches!(decoder.finish().as_slice(), [Change::Failed(_)]));
    let mut decoder = Decoder::new(true);
    assert!(matches!(
        decoder.feed(&vec![b'x'; MAX_BYTES + 1]).as_slice(),
        [Change::Failed(_)]
    ));
}

#[test]
fn raw_error_messages_are_not_exposed_and_missing_usage_is_not_estimated() {
    let mut decoder = Decoder::new(true);
    let error = event(json!({"type":"error","message":"fixture-secret-value"}));
    let changes = decoder.feed(error.as_bytes());
    assert!(!format!("{changes:?}").contains("fixture-secret-value"));
    let mut response = response();
    response["usage"] = Value::Null;
    let mut decoder = Decoder::new(false);
    decoder.feed(&serde_json::to_vec(&response).unwrap());
    let changes = decoder.finish();
    let Change::Complete(usage) = &changes[1] else {
        panic!()
    };
    assert_eq!(usage.kind, UsageKind::Unavailable);
}

#[test]
fn output_limit_scales_with_effort() {
    for (effort, limit) in [("low", 2_048), ("medium", 4_096), ("high", 8_192)] {
        let mut r = request(true);
        r.options.effort = effort.into();
        let body: Value = serde_json::from_slice(&protocol::body(&r).unwrap()).unwrap();
        assert_eq!(body["max_output_tokens"], limit, "{effort}");
    }
}

#[test]
fn incomplete_replies_keep_text_and_name_their_cause_in_both_modes() {
    for (reason, message) in [
        ("max_output_tokens", protocol::OUTPUT_LIMIT),
        ("content_filter", protocol::FILTERED),
        ("unknown", protocol::INCOMPLETE),
    ] {
        let mut incomplete = response();
        incomplete["status"] = "incomplete".into();
        incomplete["incomplete_details"] = json!({ "reason": reason });
        let expected = vec![
            Change::Delta("Hi! How can I help?".into()),
            Change::Failed(message),
        ];
        let streamed = event(json!({"type":"response.incomplete","response":incomplete}));
        let mut decoder = Decoder::new(true);
        assert_eq!(
            decoder.feed(streamed.as_bytes()),
            expected,
            "streamed {reason}"
        );
        let mut decoder = Decoder::new(false);
        decoder.feed(&serde_json::to_vec(&incomplete).unwrap());
        assert_eq!(decoder.finish(), expected, "buffered {reason}");
    }
}
