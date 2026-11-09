//! Loopback checks for the idle watchdog and transport error wording.
use super::{live_protocol::request, live_transport::receive};
use crate::provider::{
    Change, Event,
    live::{IDLE_LIMIT, INTERRUPTED, OpenAi, STALLED, UNREACHABLE},
};
use std::{
    io::Write,
    net::TcpListener,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Poll at a fixed frame time until `done` matches, waiting in real time.
fn poll_until(provider: &mut OpenAi, now: f64, done: impl Fn(&Event) -> bool) -> Vec<Event> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut all = Vec::new();
    loop {
        all.extend(provider.poll(now));
        if all.iter().any(&done) {
            return all;
        }
        assert!(Instant::now() < deadline, "local request timed out");
        thread::sleep(Duration::from_millis(5));
    }
}

fn stalled() -> Vec<Event> {
    vec![Event {
        request: 7,
        change: Change::Failed(STALLED),
    }]
}

/// A server that answers with `head` and then stays silent until released.
fn silent_server(head: &'static str) -> (String, mpsc::Sender<()>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (release, released) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        receive(&mut socket);
        socket.write_all(head.as_bytes()).unwrap();
        socket.flush().unwrap();
        let _ = released.recv_timeout(Duration::from_secs(5));
    });
    (url, release, server)
}

#[test]
fn watchdog_measures_silence_not_total_reply_time() {
    let (url, release, server) = silent_server(concat!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n",
        "Content-Length: 100000\r\nConnection: close\r\n\r\n",
        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"partial\"}\n\n",
    ));
    let mut provider = OpenAi::test_endpoint(url);
    provider.start(request(true), Some("fixture-key"), 0.0);
    // The first text arrives 200 seconds in; the reply then runs past the
    // idle limit in total and only fails once it has been silent that long.
    poll_until(&mut provider, 200.0, |e| {
        matches!(e.change, Change::Delta(_))
    });
    assert_eq!(provider.next_due(), Some(200.0 + IDLE_LIMIT));
    assert!(provider.poll(200.0 + IDLE_LIMIT - 1.0).is_empty());
    assert_eq!(provider.poll(200.0 + IDLE_LIMIT), stalled());
    assert_eq!(provider.next_due(), None);
    release.send(()).unwrap();
    server.join().unwrap();
    assert!(provider.poll(1_000.0).is_empty());
}

#[test]
fn buffered_request_without_headers_stops_after_the_idle_limit() {
    let (url, release, server) = silent_server("");
    let mut provider = OpenAi::test_endpoint(url);
    provider.start(request(false), Some("fixture-key"), 10.0);
    assert_eq!(provider.next_due(), Some(10.0 + IDLE_LIMIT));
    assert!(provider.poll(10.0 + IDLE_LIMIT - 0.5).is_empty());
    assert_eq!(provider.poll(10.0 + IDLE_LIMIT), stalled());
    release.send(()).unwrap();
    server.join().unwrap();
}

#[test]
fn transport_errors_distinguish_unreachable_from_interrupted() {
    let closed = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", closed.local_addr().unwrap());
    drop(closed);
    let mut provider = OpenAi::test_endpoint(url);
    provider.start(request(true), Some("fixture-key"), 0.0);
    let events = poll_until(&mut provider, 0.0, |e| {
        matches!(e.change, Change::Failed(_))
    });
    assert_eq!(events.last().unwrap().change, Change::Failed(UNREACHABLE));

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        receive(&mut socket);
        let head =
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: 100000\r\n\r\n";
        let part = "data: {\"type\":\"response.output_text.delta\",\"delta\":\"partial\"}\n\n";
        write!(socket, "{head}{part}").unwrap();
    });
    let mut provider = OpenAi::test_endpoint(url);
    provider.start(request(true), Some("fixture-key"), 0.0);
    let events = poll_until(&mut provider, 0.0, |e| {
        matches!(e.change, Change::Failed(_))
    });
    server.join().unwrap();
    assert_eq!(
        events.first().unwrap().change,
        Change::Delta("partial".into())
    );
    assert_eq!(events.last().unwrap().change, Change::Failed(INTERRUPTED));
}
