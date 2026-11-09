use crate::provider::{Change, Options, Prompt, Provider, Request, Role, Scenario, Scripted};

fn request(id: u64, streaming: bool, scenario: Scenario) -> Request {
    Request {
        id,
        messages: vec![Prompt {
            role: Role::User,
            text: "Show Rust 🦀".into(),
        }],
        options: Options {
            model: "démo-漢".into(),
            effort: "low".into(),
            system_prompt: "Be clear.".into(),
            streaming,
        },
        scenario,
    }
}
#[test]
fn streamed_unicode_and_buffered_replies_match_and_complete_once() {
    let mut p = Scripted::default();
    p.start(request(1, true, Scenario::Normal), 0.0);
    p.start(request(2, false, Scenario::Normal), 0.0);
    assert!(p.poll(0.44).is_empty());
    let mut texts = [String::new(), String::new()];
    let mut completed = [0, 0];
    let mut deltas = [0, 0];
    for _ in 0..30 {
        for e in p.poll(100.0) {
            let index = (e.request - 1) as usize;
            match e.change {
                Change::Delta(s) => {
                    texts[index].push_str(&s);
                    deltas[index] += 1;
                }
                Change::Complete(usage) => {
                    completed[index] += 1;
                    assert!(usage.input > 0 && usage.output > 0);
                }
                Change::Failed(e) => panic!("unexpected failure: {e}"),
            }
        }
    }
    assert_eq!(texts[0], texts[1]);
    assert!(texts[0].contains("démo-漢"));
    assert_eq!(completed, [1, 1]);
    assert!(deltas[0] > 1);
    assert_eq!(deltas[1], 1);
    assert!(p.next_due().is_none());
    assert!(p.poll(200.0).is_empty());
}
#[test]
fn cancel_is_scoped_and_produces_no_late_completion() {
    let mut p = Scripted::default();
    p.start(request(1, true, Scenario::Normal), 0.0);
    p.start(request(2, true, Scenario::Normal), 0.0);
    p.cancel(1);
    assert!(p.poll(1.0).iter().all(|e| e.request == 2));
    p.cancel(2);
    assert!(p.poll(100.0).is_empty());
    assert!(p.next_due().is_none());
}
#[test]
fn failures_are_deterministic_with_and_without_streaming() {
    for scenario in [
        Scenario::Authentication,
        Scenario::RateLimit,
        Scenario::Network,
    ] {
        for streaming in [false, true] {
            let mut p = Scripted::default();
            p.start(request(1, streaming, scenario), 0.0);
            let events = p.poll(100.0);
            assert!(events.iter().any(|e| matches!(e.change, Change::Failed(_))));
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e.change, Change::Delta(_)))
                    .count(),
                if scenario == Scenario::Network && streaming {
                    2
                } else {
                    0
                }
            );
            assert!(p.next_due().is_none());
        }
    }
}
