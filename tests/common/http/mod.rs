#![allow(dead_code)]
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub struct Server {
    pub base: String,
    calls: Arc<Mutex<HashMap<String, usize>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let calls = Arc::new(Mutex::new(HashMap::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker = {
            let (calls, stop, base) = (calls.clone(), stop.clone(), base.clone());
            thread::spawn(move || {
                let mut workers = Vec::new();
                while !stop.load(Ordering::Acquire) {
                    if let Ok((mut stream, _)) = listener.accept() {
                        let (calls, base) = (calls.clone(), base.clone());
                        workers.push(thread::spawn(move || {
                            stream.set_nonblocking(false).unwrap();
                            stream
                                .set_read_timeout(Some(Duration::from_secs(1)))
                                .unwrap();
                            let mut request = Vec::new();
                            while !request.ends_with(b"\r\n\r\n") {
                                let mut chunk = [0; 4096];
                                let Ok(n) = stream.read(&mut chunk) else {
                                    return;
                                };
                                if n == 0 {
                                    return;
                                }
                                request.extend_from_slice(&chunk[..n]);
                            }
                            let request = String::from_utf8_lossy(&request);
                            let path = request.split_whitespace().nth(1).unwrap_or("/").to_owned();
                            let count = {
                                let mut calls = calls.lock().unwrap();
                                let n = calls.entry(path.clone()).or_insert(0);
                                *n += 1;
                                *n
                            };
                            let response = response(&base, &path, count);
                            if path == "/timeout" {
                                let split =
                                    response.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
                                let _ = stream.write_all(&response[..split]);
                                thread::sleep(Duration::from_millis(350));
                                let _ = stream.write_all(&response[split..]);
                            } else {
                                let _ = stream.write_all(&response);
                            }
                        }));
                    } else {
                        thread::sleep(Duration::from_millis(2));
                    }
                }
                for worker in workers {
                    worker.join().unwrap();
                }
            })
        };
        Self {
            base,
            calls,
            stop,
            worker: Some(worker),
        }
    }
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
    pub fn calls(&self, path: &str) -> usize {
        *self.calls.lock().unwrap().get(path).unwrap_or(&0)
    }
    pub fn wait_for(&self, path: &str, count: usize) {
        let start = Instant::now();
        while self.calls(path) < count {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "waiting for {path}"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.worker.take().unwrap().join().unwrap();
    }
}
fn response(base: &str, path: &str, count: usize) -> Vec<u8> {
    if path == "/slow" && count == 1 {
        thread::sleep(Duration::from_millis(350));
    }
    let (status, body) = match path {
        "/redirect" => return format!("HTTP/1.1 302 Found\r\nLocation: {base}/png\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").into_bytes(),
        "/error" if count == 1 => (503, b"Unavailable".to_vec()),
        "/invalid" => (200, b"not an image".to_vec()),
        "/large-header" => return b"HTTP/1.1 200 OK\r\nContent-Length: 999999\r\nConnection: close\r\n\r\n".to_vec(),
        "/large-stream" => {
            let mut response = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".to_vec();
            response.extend(vec![0; 4096]);
            return response;
        }
        _ => (200, encoded(path)),
    };
    let mut response = format!(
        "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend(body);
    response
}
fn encoded(path: &str) -> Vec<u8> {
    let (width, height) = if path == "/wide" { (4096, 1) } else { (32, 16) };
    let image = image::RgbImage::from_pixel(width, height, image::Rgb([12, 150, 240]));
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(
            &mut bytes,
            if path == "/jpeg" {
                image::ImageFormat::Jpeg
            } else {
                image::ImageFormat::Png
            },
        )
        .unwrap();
    bytes.into_inner()
}
