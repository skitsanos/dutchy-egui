//! Minimal bounded SSE framing, including CRLF, comments and multiline data.
use super::MAX_BYTES;
#[derive(Default)]
pub(super) struct Sse {
    buffer: Vec<u8>,
    data: Vec<u8>,
}
impl Sse {
    pub fn feed(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, ()> {
        if self.buffer.len() + bytes.len() > MAX_BYTES {
            return Err(());
        }
        self.buffer.extend_from_slice(bytes);
        let mut events = Vec::new();
        while let Some(end) = self.buffer.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = self.buffer.drain(..=end).collect();
            self.line(line.strip_suffix(b"\n").unwrap_or(&line), &mut events)?;
        }
        Ok(events)
    }
    pub fn finish(&mut self) -> Result<Vec<Vec<u8>>, ()> {
        let mut events = Vec::new();
        let rest = std::mem::take(&mut self.buffer);
        if !rest.is_empty() {
            self.line(&rest, &mut events)?;
        }
        self.flush(&mut events);
        Ok(events)
    }
    fn line(&mut self, line: &[u8], events: &mut Vec<Vec<u8>>) -> Result<(), ()> {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            self.flush(events);
        } else if let Some(data) = line.strip_prefix(b"data:") {
            let data = data.strip_prefix(b" ").unwrap_or(data);
            if self.data.len() + data.len() + 1 > MAX_BYTES {
                return Err(());
            }
            if !self.data.is_empty() {
                self.data.push(b'\n');
            }
            self.data.extend_from_slice(data);
        }
        Ok(())
    }
    fn flush(&mut self, events: &mut Vec<Vec<u8>>) {
        if !self.data.is_empty() {
            events.push(std::mem::take(&mut self.data));
        }
    }
}
