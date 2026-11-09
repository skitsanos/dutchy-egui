//! Conversation lifecycle and stale-event isolation around the provider protocol.
mod model;
use crate::{
    config::Config,
    provider::{Change, Event, Options, Prompt, Request, Role, Router, Scenario},
};
use model::Ids;
pub(crate) use model::{Conversation, MAX_PROMPT, Message, Status};

pub(crate) struct Chat {
    pub conversations: Vec<Conversation>,
    pub selected: u64,
    provider: Router,
    ids: Ids,
}
impl Default for Chat {
    fn default() -> Self {
        let mut chat = Self {
            conversations: Vec::new(),
            selected: 0,
            provider: Router::default(),
            ids: Ids::default(),
        };
        chat.new_conversation().expect("initial IDs are available");
        chat
    }
}
impl Chat {
    pub fn context(&mut self, context: egui::Context) {
        self.provider.context(context);
    }
    pub fn current(&self) -> &Conversation {
        self.conversations
            .iter()
            .find(|c| c.id == self.selected)
            .expect("selected conversation exists")
    }
    pub fn current_mut(&mut self) -> &mut Conversation {
        self.conversations
            .iter_mut()
            .find(|c| c.id == self.selected)
            .expect("selected conversation exists")
    }
    pub fn new_conversation(&mut self) -> Result<(), &'static str> {
        let id = self.ids.take()?;
        self.conversations.push(Conversation {
            id,
            title: "New conversation".into(),
            draft: String::new(),
            messages: Vec::new(),
            active: None,
            usage: None,
        });
        self.selected = id;
        Ok(())
    }
    pub fn select(&mut self, id: u64) {
        if self.conversations.iter().any(|c| c.id == id) {
            self.selected = id;
        }
    }
    pub fn delete_current(&mut self) -> Result<(), &'static str> {
        let id = self.selected;
        if self.conversations.len() == 1 {
            self.new_conversation()?;
        }
        self.remove(id);
        Ok(())
    }
    pub fn remove(&mut self, id: u64) {
        if self.conversations.len() <= 1 {
            return;
        }
        if let Some(run) = self
            .conversations
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.active)
        {
            self.provider.cancel(run);
        }
        self.conversations.retain(|c| c.id != id);
        if self.selected == id {
            self.selected = self.conversations[0].id;
        }
    }
    pub fn send(
        &mut self,
        config: &Config,
        scenario: Scenario,
        now: f64,
    ) -> Result<(), &'static str> {
        let current = self.current();
        if current.active.is_some() {
            return Err("A reply is already running in this conversation.");
        }
        if current.draft.trim().is_empty() {
            return Err("Write a message first.");
        }
        if current.draft.chars().count() > MAX_PROMPT {
            return Err("Keep a message within 4,000 characters.");
        }
        let user = self.ids.take()?;
        let assistant = self.ids.take()?;
        let request = self.ids.take()?;
        let current = self.current_mut();
        let text = std::mem::take(&mut current.draft).trim().to_owned();
        if current.messages.is_empty() {
            current.title = title(&text);
        }
        current.messages.push(Message {
            id: user,
            role: Role::User,
            text,
            status: Status::Complete,
            provider: config.provider,
        });
        current.messages.push(Message {
            id: assistant,
            role: Role::Assistant,
            text: String::new(),
            status: Status::Receiving,
            provider: config.provider,
        });
        self.start(request, config, scenario, now);
        Ok(())
    }
    pub fn retry(
        &mut self,
        config: &Config,
        scenario: Scenario,
        now: f64,
    ) -> Result<(), &'static str> {
        if !self.current().retryable() {
            return Err("There is no reply to retry.");
        }
        let request = self.ids.take()?;
        let last = self
            .current_mut()
            .messages
            .last_mut()
            .expect("retry has a reply");
        last.text.clear();
        last.status = Status::Receiving;
        last.provider = config.provider;
        self.start(request, config, scenario, now);
        Ok(())
    }
    fn start(&mut self, request: u64, config: &Config, scenario: Scenario, now: f64) {
        let current = self.current_mut();
        current.active = Some(request);
        current.usage = None;
        let messages = current.messages[..current.messages.len() - 1]
            .iter()
            .filter(|m| m.status == Status::Complete)
            .map(|m| Prompt {
                role: m.role,
                text: m.text.clone(),
            })
            .collect();
        self.provider.start(
            Request {
                id: request,
                messages,
                scenario,
                options: Options {
                    model: config.model.clone(),
                    effort: config.effort.clone(),
                    system_prompt: config.system_prompt.clone(),
                    streaming: config.streaming,
                },
            },
            config.provider,
            config.credential(),
            now,
        );
    }
    pub fn stop(&mut self) -> bool {
        let Some(request) = self.current_mut().active.take() else {
            return false;
        };
        self.provider.cancel(request);
        if let Some(message) = self.current_mut().messages.last_mut() {
            message.status = Status::Stopped;
        }
        true
    }
    pub fn poll(&mut self, now: f64) -> Vec<String> {
        self.provider
            .poll(now)
            .into_iter()
            .filter_map(|event| self.apply(event))
            .collect()
    }
    pub fn next_due(&self) -> Option<f64> {
        self.provider.next_due()
    }
    pub fn apply(&mut self, event: Event) -> Option<String> {
        let conversation = self
            .conversations
            .iter_mut()
            .find(|c| c.active == Some(event.request))?;
        let message = conversation.messages.last_mut()?;
        match event.change {
            Change::Delta(text) => {
                message.text.push_str(&text);
                None
            }
            Change::Complete(usage) => {
                message.status = Status::Complete;
                conversation.active = None;
                conversation.usage = Some(usage);
                Some(format!("Reply complete in {}.", conversation.title))
            }
            Change::Failed(error) => {
                message.status = Status::Failed(error);
                conversation.active = None;
                Some(format!("Reply failed in {}.", conversation.title))
            }
        }
    }
}
fn title(text: &str) -> String {
    let first = text.lines().next().unwrap_or(text);
    let mut title: String = first.chars().take(36).collect();
    if first.chars().count() > 36 {
        title.push('…');
    }
    title
}
