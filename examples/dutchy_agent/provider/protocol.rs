//! Responses JSON and SSE decoding. Transport chunks are not event boundaries.
mod sse;
use super::{Change, Request, Role, Usage, UsageKind};
use serde_json::{Value, json};
use sse::Sse;

pub(crate) const MAX_BYTES: usize = 2 * 1024 * 1024;
const INVALID: &str = "OpenAI returned an invalid response. Please retry.";
const TRUNCATED: &str = "The OpenAI connection ended before the reply completed. Please retry.";
pub(crate) const OUTPUT_LIMIT: &str = "OpenAI reached this example's output limit before finishing. Choose a lower effort or ask for a shorter reply.";
pub(crate) const FILTERED: &str = "OpenAI's content filter stopped this reply.";
pub(crate) const INCOMPLETE: &str = "OpenAI returned an incomplete reply. Please retry.";

pub(crate) fn body(request: &Request) -> Result<Vec<u8>, &'static str> {
    if request.options.model.trim().is_empty() {
        return Err("Set a model name in Chat settings.");
    }
    let input: Vec<_> = request
        .messages
        .iter()
        .map(|m| {
            json!({
                "role": if m.role==Role::User {"user"}else{"assistant"}, "content":m.text,
            })
        })
        .collect();
    let body = serde_json::to_vec(&json!({
        "model":request.options.model,"instructions":request.options.system_prompt,
        "input":input,"reasoning":{"effort":request.options.effort},
        "stream":request.options.streaming,"store":false,
        "max_output_tokens":output_limit(&request.options.effort),
    }))
    .map_err(|_| INVALID)?;
    if body.len() > MAX_BYTES {
        return Err("This conversation is too large. Start a new conversation.");
    }
    Ok(body)
}

/// Reasoning tokens count toward the output limit, so higher effort gets more room.
fn output_limit(effort: &str) -> u32 {
    match effort {
        "high" => 8_192,
        "medium" => 4_096,
        _ => 2_048,
    }
}

pub(crate) struct Decoder {
    streaming: bool,
    sse: Sse,
    buffer: Vec<u8>,
    terminal: bool,
    text_seen: bool,
    output_bytes: usize,
}
impl Decoder {
    pub fn new(streaming: bool) -> Self {
        Self {
            streaming,
            sse: Sse::default(),
            buffer: Vec::new(),
            terminal: false,
            text_seen: false,
            output_bytes: 0,
        }
    }
    pub fn terminal(&self) -> bool {
        self.terminal
    }
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Change> {
        if self.terminal {
            return Vec::new();
        }
        if !self.streaming {
            if self.buffer.len() + bytes.len() > MAX_BYTES {
                return self.fail(INVALID);
            }
            self.buffer.extend_from_slice(bytes);
            return Vec::new();
        }
        match self.sse.feed(bytes) {
            Ok(events) => self.events(events),
            Err(()) => self.fail(INVALID),
        }
    }
    pub fn finish(&mut self) -> Vec<Change> {
        if self.terminal {
            return Vec::new();
        }
        let mut changes = if self.streaming {
            match self.sse.finish() {
                Ok(events) => self.events(events),
                Err(()) => self.fail(INVALID),
            }
        } else {
            match serde_json::from_slice::<Value>(&self.buffer) {
                Ok(response) => self.complete(&response),
                Err(_) => self.fail(INVALID),
            }
        };
        if !self.terminal {
            changes.extend(self.fail(TRUNCATED));
        }
        changes
    }
    fn events(&mut self, events: Vec<Vec<u8>>) -> Vec<Change> {
        let mut changes = Vec::new();
        for data in events {
            if self.terminal {
                break;
            }
            if data == b"[DONE]" {
                changes.extend(self.fail(TRUNCATED));
                break;
            }
            let value = match serde_json::from_slice::<Value>(&data) {
                Ok(value) => value,
                Err(_) => {
                    changes.extend(self.fail(INVALID));
                    break;
                }
            };
            changes.extend(self.event(&value));
        }
        changes
    }
    fn event(&mut self, event: &Value) -> Vec<Change> {
        match event["type"].as_str() {
            Some("response.output_text.delta" | "response.refusal.delta") => {
                match event["delta"].as_str() {
                    Some(text) => self.delta(text),
                    None => self.fail(INVALID),
                }
            }
            Some("response.completed" | "response.incomplete") => self.complete(&event["response"]),
            Some("response.failed" | "error") => self
                .fail("OpenAI could not complete this reply. Check the model settings and retry."),
            Some(_) => Vec::new(),
            None => self.fail(INVALID),
        }
    }
    fn delta(&mut self, text: &str) -> Vec<Change> {
        if self.output_bytes + text.len() > MAX_BYTES {
            return self.fail("The reply exceeded this example's size limit.");
        }
        self.output_bytes += text.len();
        self.text_seen |= !text.is_empty();
        if text.is_empty() {
            Vec::new()
        } else {
            vec![Change::Delta(text.to_owned())]
        }
    }
    /// Finish from a final response object, in either mode. Text that only
    /// the final object carries is kept, even when the reply is incomplete.
    fn complete(&mut self, response: &Value) -> Vec<Change> {
        let mut changes = Vec::new();
        if !self.text_seen {
            changes.extend(self.delta(&text(response)));
        }
        if self.terminal {
            return changes;
        }
        changes.extend(match response["status"].as_str() {
            Some("completed") if self.text_seen => {
                self.terminal = true;
                vec![Change::Complete(usage(response))]
            }
            Some("completed") => self.fail("OpenAI returned no text response. Please retry."),
            Some("incomplete") => self.fail(incomplete(response)),
            _ => self.fail(INCOMPLETE),
        });
        changes
    }
    fn fail(&mut self, message: &'static str) -> Vec<Change> {
        self.terminal = true;
        vec![Change::Failed(message)]
    }
}

fn incomplete(response: &Value) -> &'static str {
    match response["incomplete_details"]["reason"].as_str() {
        Some("max_output_tokens") => OUTPUT_LIMIT,
        Some("content_filter") => FILTERED,
        _ => INCOMPLETE,
    }
}
fn text(response: &Value) -> String {
    let mut text = String::new();
    for item in response["output"].as_array().into_iter().flatten() {
        if item["type"] != "message" || item["role"] != "assistant" {
            continue;
        }
        for part in item["content"].as_array().into_iter().flatten() {
            let value = match part["type"].as_str() {
                Some("output_text") => part["text"].as_str(),
                Some("refusal") => part["refusal"].as_str(),
                _ => None,
            };
            if let Some(value) = value {
                text.push_str(value);
            }
        }
    }
    text
}
fn usage(response: &Value) -> Usage {
    let input = response["usage"]["input_tokens"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok());
    let output = response["usage"]["output_tokens"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok());
    Usage {
        input: input.unwrap_or(0),
        output: output.unwrap_or(0),
        kind: if input.is_some() && output.is_some() {
            UsageKind::Reported
        } else {
            UsageKind::Unavailable
        },
        response_id: response["id"].as_str().map(str::to_owned),
        model: response["model"].as_str().map(str::to_owned),
    }
}
