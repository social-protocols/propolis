use async_trait::async_trait;
use serde::Serialize;

/// Contains the information to identify an ai environment
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
pub struct AiEnvInfo {
    /// Name of the provider. e.g. "openai"
    pub name: String,
    /// Actual model used
    pub model: String,
    /// Model to use for checking / moderating
    pub check_model: Option<String>,
}

/// Contains the information to identify a used prompt
#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
pub struct PromptInfo {
    /// Name of the prompt function
    pub name: String,
    /// Used version
    pub version: u16,
}

/// Contains the result of checking a prompt against e.g. a moderation api
pub enum CheckResult {
    /// Everything OK
    Ok,
    /// Flagged as inappropriate
    Flagged(String),
}

impl From<AiEnvInfo> for String {
    fn from(value: AiEnvInfo) -> Self {
        format!("{},{}", value.model, value.name)
    }
}

#[async_trait]
pub trait AiEnv {
    /// Returns information on the ai environment
    fn info(&self) -> AiEnvInfo;

    /// Check the prompt against e.g. moderation api
    async fn check_prompt<Prompt: AiPrompt>(
        &self,
        r: &Prompt
    ) -> anyhow::Result<CheckResult>;

    /// Run prompt against actual model
    async fn send_prompt<Prompt: AiPrompt>(
        &self,
        r: &Prompt,
    ) -> anyhow::Result<Prompt::PromptResult>;
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
    pub fn system(s: &str) -> Self {
        AiMessage {
            role: AiRole::System,
            content: s.to_string(),
        }
    }
    pub fn user(s: &str) -> Self {
        AiMessage {
            role: AiRole::User,
            content: s.to_string(),
        }
    }
    pub fn assistant(s: &str) -> Self {
        AiMessage {
            role: AiRole::Assistant,
            content: s.to_string(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct PromptResponse {
    pub env_info: AiEnvInfo,
    pub prompt_info: PromptInfo,
    /// Response content
    pub content: String,
    /// Amount of tokens used for the input prompt
    pub prompt_tokens: i64,
    /// Amount of tokens used for output completion
    pub completion_tokens: i64,
    /// Total amount of tokens used
    pub total_tokens: i64,
}

/// Trait for something that is a response of a prompt
pub trait AsPromptResponse: Serialize {
    /// Response content
    fn content(&self) -> &str;
    /// Amount of tokens used for the input prompt
    fn prompt_tokens(&self) -> i64;
    /// Amount of tokens used for output completion
    fn completion_tokens(&self) -> i64;
    /// Total amount of tokens used
    fn total_tokens(&self) -> i64;
}

impl AsPromptResponse for PromptResponse {
    fn content(&self) -> &str {
        self.content.as_str()
    }
    fn prompt_tokens(&self) -> i64 {
        self.prompt_tokens
    }
    fn completion_tokens(&self) -> i64 {
        self.completion_tokens
    }
    fn total_tokens(&self) -> i64 {
        self.total_tokens
    }
}

/// Represents a prompt to send to an ai plus its postprocessing via handler
pub trait AiPrompt: Send + Sync {
    // Require the result to be serializable so we can store it in the db
    type PromptResult: serde::Serialize;

    fn info(&self) -> PromptInfo {
        PromptInfo {
            name: self.name().into(),
            version: self.version(),
        }
    }
    /// Used to disambiguate different prompts. Should be unique for every use-case
    fn name(&self) -> &str;
    /// Version to disambiguate different versions of one prompt. Used for regenerations
    fn version(&self) -> u16;
    /// String that will be use to prime the ai
    fn primer(&self) -> Vec<AiMessage>;
    /// Handle the response from the ai
    fn handle_response(&self, r: PromptResponse) -> anyhow::Result<Self::PromptResult>;
}
