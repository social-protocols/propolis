use super::api::{AiMessage, AiPrompt, PromptResponse};

pub struct GenericPrompt {
    pub name: String,
    pub version: u16,
    pub primer: Vec<AiMessage>,
    pub handler: fn(String) -> String,
}

impl AiPrompt for GenericPrompt {
    type PromptResult = PromptResponse;

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn primer(&self) -> Vec<AiMessage> {
        self.primer.clone()
    }

    fn handle_response(&self, r: PromptResponse) -> Self::PromptResult {
        PromptResponse {
            content: (self.handler)(r.content),
            completion_tokens: r.completion_tokens,
            prompt_tokens: r.prompt_tokens,
            total_tokens: r.total_tokens,
        }
        
    }
}
