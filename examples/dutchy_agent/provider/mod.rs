//! Shared lifecycle for explicit live and scripted providers.
pub(crate) mod live;
pub(crate) mod protocol;
mod router;
mod script;
mod scripted;
pub(crate) use router::Router;
pub(crate) use scripted::Scripted;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    OpenAi,
    #[default]
    Mock,
}
impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Mock => "Offline mock",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    User,
    Assistant,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Prompt {
    pub role: Role,
    pub text: String,
}
#[derive(Clone, Debug)]
pub(crate) struct Options {
    pub model: String,
    pub effort: String,
    pub system_prompt: String,
    pub streaming: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct Request {
    pub id: u64,
    pub messages: Vec<Prompt>,
    pub options: Options,
    pub scenario: Scenario,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Scenario {
    #[default]
    Normal,
    Authentication,
    RateLimit,
    Network,
}
impl Scenario {
    pub const ALL: [Self; 4] = [
        Self::Normal,
        Self::Authentication,
        Self::RateLimit,
        Self::Network,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Successful reply",
            Self::Authentication => "Authentication error",
            Self::RateLimit => "Rate limit",
            Self::Network => "Interrupted connection",
        }
    }
    pub fn error(self) -> Option<&'static str> {
        match self {
            Self::Normal => None,
            Self::Authentication => {
                Some("Simulated authentication error. No credentials were sent.")
            }
            Self::RateLimit => Some("Simulated rate limit. Choose Successful reply, then retry."),
            Self::Network => {
                Some("Simulated connection interruption. Your partial reply is preserved.")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UsageKind {
    Estimated,
    Reported,
    #[default]
    Unavailable,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Usage {
    pub input: usize,
    pub output: usize,
    pub kind: UsageKind,
    pub response_id: Option<String>,
    pub model: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Change {
    Delta(String),
    Complete(Usage),
    Failed(&'static str),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub request: u64,
    pub change: Change,
}
pub(crate) trait Provider {
    fn start(&mut self, request: Request, now: f64);
    fn poll(&mut self, now: f64) -> Vec<Event>;
    fn cancel(&mut self, request: u64);
    fn next_due(&self) -> Option<f64>;
}
