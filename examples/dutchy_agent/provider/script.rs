//! Fictional replies authored for this example; no embedded corpus or remote data.
use super::{Options, Prompt, Role};

pub(crate) const RUST_CODE: &str =
    "fn main() {\n    let message = \"Make something sharp.\";\n    println!(\"{message}\");\n}\n";

pub(super) fn reply(messages: &[Prompt], options: &Options) -> String {
    let rust = messages
        .iter()
        .rev()
        .find(|p| p.role == Role::User)
        .is_some_and(|p| p.text.to_lowercase().contains("rust"));
    let heading = if rust {
        "A small Rust sketch"
    } else {
        "A clear starting point"
    };
    format!(
        "### {heading}\n\nThis is a **scripted offline reply**, not a model response.\n\n\
         1. Keep the first step small.\n2. Make state explicit.\n3. Check the result.\n\n\
         ```rust\n{RUST_CODE}```\n\n\
         > [!NOTE]\n> You can copy the code, stop a reply, or continue in another conversation.\n\n\
         | Step | Result |\n| --- | --- |\n| Build | A small example |\n| Verify | A repeatable check |\n\n\
         [About the offline provider](mock://about)\n\n\
         The configured model is `{}` with `{}` effort. The system prompt contains {} characters.",
        safe(&options.model),
        safe(&options.effort),
        options.system_prompt.chars().count()
    )
}

fn safe(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
        .collect()
}

pub(super) fn chunks(text: &str, streaming: bool) -> std::collections::VecDeque<String> {
    if !streaming {
        return [text.to_owned()].into();
    }
    let mut chunks = std::collections::VecDeque::new();
    let mut chunk = String::new();
    for (index, character) in text.chars().enumerate() {
        chunk.push(character);
        if index % 24 == 23 {
            chunks.push_back(std::mem::take(&mut chunk));
        }
    }
    if !chunk.is_empty() {
        chunks.push_back(chunk);
    }
    chunks
}
