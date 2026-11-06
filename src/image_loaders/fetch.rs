use super::{UrlImageOptions, decode};
use ehttp::streaming::Part;
use std::{
    ops::ControlFlow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

#[derive(Default)]
pub(super) struct Request {
    pub result: Mutex<Option<Result<egui::ColorImage, String>>>,
    cancelled: AtomicBool,
}
impl Request {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
struct Flight {
    active: Arc<AtomicUsize>,
    ctx: egui::Context,
}
impl Drop for Flight {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::AcqRel);
        self.ctx.request_repaint();
    }
}
pub(super) fn start(
    ctx: egui::Context,
    uri: &str,
    state: Arc<Request>,
    active: Arc<AtomicUsize>,
    options: UrlImageOptions,
) {
    let max_side = ctx.input(|input| input.max_texture_side.min(u32::MAX as usize) as u32);
    let flight = Flight { active, ctx };
    let bytes = Mutex::new(Vec::new());
    let mut request = ehttp::Request::get(uri);
    request.timeout = Some(options.response_body_timeout);
    request.headers.insert("Accept", "image/png, image/jpeg");
    ehttp::streaming::fetch(request, move |part| {
        // Keep the concurrency permit until the transport releases its callback.
        let _permit = &flight;
        if state.cancelled.load(Ordering::Acquire) {
            return ControlFlow::Break(());
        }
        let result = consume(part, &bytes, options, max_side);
        match result {
            Ok(None) => ControlFlow::Continue(()),
            result => {
                if !state.cancelled.load(Ordering::Acquire) {
                    *state.result.lock().unwrap() = Some(result.map(Option::unwrap));
                    flight.ctx.request_repaint();
                }
                ControlFlow::Break(())
            }
        }
    });
}
fn consume(
    part: ehttp::Result<Part>,
    bytes: &Mutex<Vec<u8>>,
    options: UrlImageOptions,
    max_side: u32,
) -> Result<Option<egui::ColorImage>, String> {
    match part? {
        Part::Response(response) => {
            if !response.ok {
                return Err(format!("HTTP {}", response.status));
            }
            if response
                .headers
                .get("content-length")
                .and_then(|n| n.parse::<usize>().ok())
                .is_some_and(|length| length > options.max_download_bytes)
            {
                return Err("Image exceeds the download byte limit".into());
            }
            Ok(None)
        }
        Part::Chunk(chunk) => {
            let mut bytes = bytes.lock().unwrap();
            if chunk.is_empty() {
                return decode::decode(&bytes, options, max_side).map(Some);
            }
            if chunk.len() > options.max_download_bytes.saturating_sub(bytes.len()) {
                return Err("Image exceeds the download byte limit".into());
            }
            bytes.extend_from_slice(&chunk);
            Ok(None)
        }
    }
}
