use crate::structs::Statement;

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

/// Computes the big five personality traits for a statement
pub fn bfp(s: &Statement) -> GenericPrompt {
    GenericPrompt {
        name: "BFP".to_string(),
        version: 4,
        handler: |s| s,
        primer: vec![
            AiMessage::system(
                "Categorize via big five personality traits psychological test. No notes.",
            ),
            AiMessage::user("I enjoy trying new foods."),
            AiMessage::assistant("openness-to-experience: medium"),
            AiMessage::user("I like talking to people."),
            AiMessage::assistant("extraversion: high"),
            AiMessage::user("Refugees in germany behave badly and should be sanctioned."),
            AiMessage::assistant("agreeableness: low"),
            // the actual prediction
            AiMessage::user(s.text.as_str()),
        ],
    }
}

/// Tries to determine the category that a statement falls into
pub fn statement_category(s: &Statement) -> GenericPrompt {
    GenericPrompt {
        name: "statement_category".to_string(),
        version: 3,
        handler: |s| s,
        primer: vec![
            AiMessage::system("Determine if a statement is political or personal."),
            AiMessage::user("German parliament is too big."),
            AiMessage::assistant("political"),
            AiMessage::user("Social media is bad for mental health."),
            AiMessage::assistant("personal"),
            // the actual prediction
            AiMessage::user(s.text.as_str()),
        ],
    }
}
