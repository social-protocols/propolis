use anyhow::anyhow;
use async_trait::async_trait;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    embeddings::Embeddings,
    moderations::{Categories, ModerationBuilder, ModerationResult},
};
use serde_json::json;

use crate::api::{AsEmbeddingEnv, CheckResult, EmbedResult, Embedding};

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
#[derive(PartialEq, Eq, Debug)]
pub struct OpenAiEnv {
    pub model: &'static str,       // e.g. gpt-3.5-turbo, text-davinci-003, etc.
    pub check_model: &'static str, // e.g. text-moderation-stable, text-moderation-latest etc.
}

impl OpenAiEnv {
    pub fn from(model: OpenAiModel) -> Self {
        Self {
            model: model.into(),
            check_model: "text-moderation-stable",
        }
    }
}

#[async_trait]
impl AiEnv for OpenAiEnv {
    fn info(&self) -> AiEnvInfo {
        AiEnvInfo {
            name: "openai".into(),
            model: self.model.into(),
            check_model: Some(self.check_model.into()),
        }
    }

    async fn check_prompt<Prompt: AiPrompt>(&self, prompt: &Prompt) -> anyhow::Result<CheckResult> {
        let data = prompt.primer().iter().fold(String::new(), |result, msg| {
            let content = &msg.content;
            format!("{result}\n{content}")
        });
        let result = ModerationBuilder::default()
            .input(data)
            .model(self.check_model)
            .create()
            .await??;
        match result.results.as_slice() {
            [ModerationResult {
                flagged,
                categories:
                    Categories {
                        hate,
                        hate_threatening,
                        self_harm,
                        sexual,
                        sexual_minors,
                        violence,
                        violence_graphic,
                        ..
                    },
                ..
            }, ..] => Ok(match flagged {
                false => CheckResult::Ok,
                true => CheckResult::Flagged(serde_json::to_string(&json!([{
                    "hate": hate,
                    "hate_threatening": hate_threatening,
                    "self_harm": self_harm,
                    "sexual": sexual,
                    "sexual_minors": sexual_minors,
                    "violence": violence,
                    "violence_graphic": violence_graphic,
                }]))?),
            }),
            _ => Err(anyhow!("OpenAI moderation API yielded an empty result")),
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

        prompt.handle_response(PromptResponse {
            env_info: self.info(),
            prompt_info: prompt.info(),
            content: raw_result,
            completion_tokens: c.into(),
            prompt_tokens: p.into(),
            total_tokens: t.into(),
        })
    }
}

#[async_trait]
impl AsEmbeddingEnv for OpenAiEnv {
    async fn embed(&self, stmts: &[&str]) -> anyhow::Result<EmbedResult> {
        let model = "text-embedding-ada-002";
        let embeddings = Embeddings::create(model, stmts.to_vec(), "").await??;
        let mut data: Vec<Embedding> = vec![];
        for e in embeddings.data {
            data.push(Embedding { values: e.vec });
        }

        Ok(EmbedResult {
            data,
            prompt_tokens: embeddings.usage.prompt_tokens,
            total_tokens: embeddings.usage.total_tokens,
        })
    }
}

/// Set the key to be used when next sending a request to openai
pub fn set_key(s: String) {
    openai::set_key(s);
}
