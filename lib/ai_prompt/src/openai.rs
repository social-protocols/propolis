use anyhow::Context;
use async_trait::async_trait;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use std::env;

use super::api::{AiEnv, AiEnvInfo, AiPrompt, AiRole, PromptResponse};

impl From<AiRole> for ChatCompletionMessageRole {
    fn from(value: AiRole) -> Self {
        match value {
            AiRole::System => Self::System,
            AiRole::User => Self::User,
            AiRole::Assistant => Self::Assistant,
        }
    }
}

pub enum OpenAiModel {
    Gpt4,
    Gpt35Turbo,
    Gpt35TextDavinci003,
    Gpt35TextDavinci002,
    Gpt35CodeDavinci002,
}

impl From<OpenAiModel> for &str {
    fn from(value: OpenAiModel) -> Self {
        match value {
            OpenAiModel::Gpt4 => "gpt-4",
            OpenAiModel::Gpt35Turbo => "gpt-3.5-turbo",
            OpenAiModel::Gpt35TextDavinci003 => "text-davinci-003",
            OpenAiModel::Gpt35TextDavinci002 => "text-davinci-002",
            OpenAiModel::Gpt35CodeDavinci002 => "code-davinci-002",
        }
    }
}

/// Environment for running stuff against OpenAI models
pub struct OpenAiEnv {
    pub model: &'static str, // e.g. gpt-3.5-turbo, text-davinci-003, etc.
}

impl OpenAiEnv {
    pub fn from(model: OpenAiModel) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait]
impl AiEnv for OpenAiEnv {
    fn info(&self) -> AiEnvInfo {
        AiEnvInfo {
            name: "openai".into(),
            model: self.model.into(),
        }
    }

    async fn send_prompt<Prompt: AiPrompt>(
        &self,
        prompt: &Prompt,
    ) -> anyhow::Result<Prompt::PromptResult> {
        let mut messages: Vec<ChatCompletionMessage> = vec![];
        for m in prompt.primer() {
            messages.push(ChatCompletionMessage {
                role: m.role.into(),
                content: m.content,
                name: None,
            })
        }

        let result = ChatCompletion::builder(self.model, messages.to_owned())
            .create()
            .await?
            .unwrap();
        let (c, p, t) = result.usage.map_or((0, 0, 0), |us| {
            (us.completion_tokens, us.prompt_tokens, us.total_tokens)
        });
        let raw_result = result.choices.first().unwrap().message.content.to_owned();

        Ok(prompt.handle_response(PromptResponse {
            env_info: self.info(),
            prompt_info: prompt.info(),
            content: raw_result,
            completion_tokens: c.into(),
            prompt_tokens: p.into(),
            total_tokens: t.into(),
        }))
    }
}

pub async fn setup_openai() -> anyhow::Result<()> {
    set_key(env::var("OPENAI_API_KEY").context("OPENAI_API_KEY environment variable not found.")?);
    Ok(())
}
