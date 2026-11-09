//! Real loopback HTTP tests, never developer credentials or external network.
use super::live_protocol::{request, response};
use crate::provider::{Change, Event, live::OpenAi};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn receive(socket: &mut TcpStream) -> String {
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let mut buffer = [0; 2048];
    loop {
        let n = socket.read(&mut buffer).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&buffer[..n]);
        let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&bytes[..end]);
        let length = headers
            .lines()
            .find_map(|l| {
                l.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(|s| s.trim().parse::<usize>().unwrap())
            })
            .unwrap();
        if bytes.len() >= end + 4 + length {
            return String::from_utf8(bytes).unwrap();
        }
    }
}
fn collect(provider: &mut OpenAi) -> Vec<Event> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut all = Vec::new();
    loop {
        all.extend(provider.poll());
        if all
            .iter()
            .any(|e| matches!(e.change, Change::Complete(_) | Change::Failed(_)))
        {
            return all;
        }
        assert!(Instant::now() < deadline, "local request timed out");
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn live_transport_sends_authorized_json_and_handles_both_response_modes() {
    for streaming in [true, false] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1/responses", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let input = receive(&mut socket);
            assert!(input.starts_with("POST /v1/responses "));
            let types: Vec<_> = input
                .lines()
                .filter(|line| line.to_ascii_lowercase().starts_with("content-type:"))
                .collect();
            assert_eq!(types.len(), 1);
            assert!(types[0].ends_with("application/json"));
            assert!(
                input
                    .to_ascii_lowercase()
                    .contains("authorization: bearer fixture-key")
            );
            let body = if streaming {
                format!(
                    "data: {}\n\ndata: {}\n\n",
                    serde_json::json!({"type":"response.output_text.delta","delta":"Hi! How can I help?"}),
                    serde_json::json!({"type":"response.completed","response":response()})
                )
            } else {
                response().to_string()
            };
            let content = if streaming {
                "text/event-stream"
            } else {
                "application/json"
            };
            write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: {content}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            input
        });
        let mut provider = OpenAi::test_endpoint(url);
        provider.start(request(streaming), Some("fixture-key"));
        let events = collect(&mut provider);
        assert!(events.iter().all(|e| e.request == 7));
        assert!(
            events
                .iter()
                .any(|e| matches!(&e.change,Change::Delta(s)if s=="Hi! How can I help?"))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e.change, Change::Complete(_)))
        );
        let input = server.join().unwrap();
        let body: serde_json::Value =
            serde_json::from_str(input.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["model"], "gpt-6-luna");
        assert_eq!(body["reasoning"]["effort"], "low");
    }
}

#[test]
fn http_failures_do_not_echo_server_bodies_or_credentials() {
    for status in [401, 429, 500] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            receive(&mut socket);
            let body = "fixture-secret-key-value";
            let _ = write!(
                socket,
                "HTTP/1.1 {status} Error\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        });
        let mut provider = OpenAi::test_endpoint(url);
        provider.start(request(true), Some("fixture-key"));
        let events = collect(&mut provider);
        assert!(matches!(
            events.as_slice(),
            [Event {
                change: Change::Failed(_),
                ..
            }]
        ));
        assert!(!format!("{events:?}").contains("fixture-secret"));
        server.join().unwrap();
    }
}

#[test]
fn cancel_drops_delivery_while_the_network_reader_unwinds() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (release_tx, release_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        receive(&mut socket);
        let first = "data: {\"type\":\"response.output_text.delta\",\"delta\":\"partial\"}\n\n";
        let rest = format!(
            "data: {}\n\n",
            serde_json::json!({"type":"response.completed","response":response()})
        );
        write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{first}",first.len()+rest.len()).unwrap();
        socket.flush().unwrap();
        release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let _ = socket.write_all(rest.as_bytes());
    });
    let mut provider = OpenAi::test_endpoint(url);
    provider.start(request(true), Some("fixture-key"));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if provider
            .poll()
            .iter()
            .any(|e| matches!(e.change, Change::Delta(_)))
        {
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    provider.cancel(7);
    release_tx.send(()).unwrap();
    server.join().unwrap();
    assert!(provider.poll().is_empty());
}
