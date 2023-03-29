use std::vec;

use anyhow::anyhow;
use async_trait::async_trait;

#[async_trait]
pub trait AiEnv {
    async fn send_prompt<Prompt : AiPrompt>(&self, r : &Prompt) -> anyhow::Result<Prompt::PromptResult>;
}

#[derive(Clone)]
pub enum AiRole {
    System,
    User,
    Assistant,
}

#[derive(Clone)]
pub struct AiMessage {
    pub role: AiRole,
    pub content: String,
}

impl AiMessage {
    pub fn system(s : &str) -> Self {
        AiMessage { role: AiRole::System, content: s.to_string() }
    }
    pub fn user(s : &str) -> Self {
        AiMessage { role: AiRole::User, content: s.to_string() }
    }
    pub fn assistant(s : &str) -> Self {
        AiMessage { role: AiRole::Assistant, content: s.to_string() }
    }
}

/// Represents a prompt to send to an ai plus its postprocessing via handler
pub trait AiPrompt: Send + Sync {
    // Require the result to be serializable so we can store it in the db
    type PromptResult : serde::Serialize;

    /// Used to disambiguate different prompts. Should be unique for every use-case
    fn name(&self) -> &str;
    /// Version to disambiguate different versions of one prompt. Used for regenerations
    fn version(&self) -> u16;
    /// String that will be use to prime the ai
    fn primer(&self) -> Vec<AiMessage>;
    /// Handle the response from the ai
    fn handle_response(&self, r: String) -> Self::PromptResult;
}
