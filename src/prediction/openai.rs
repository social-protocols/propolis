use anyhow::anyhow;
use async_trait::async_trait;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use serde_json::Value;
use std::env;

use crate::prediction::data;

use super::api::{AiEnv, AiPrompt, AiRole, PromptResponse};

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
    fn name(&self) -> String {
        format!("openai--{}", self.model).to_string()
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
            content: raw_result,
            completion_tokens: c.into(),
            prompt_tokens: p.into(),
            total_tokens: t.into(),
        }))
    }
}

pub fn determine_topics() {
    r###"
Given these statements:

1 | Deutschland hat mit Angela Merkel heute mehr Probleme als Lösungen parat
2 | Meine Eltern haben mich nicht gut erzogen
3 | Liebe ist schmerzvoll
4 | Zu viel fernsehen führt zu Verblödung
5 | Die Ukraine wird durch Putin zerstört


put them into one of these categories:
1 | Politics
2 | Psychology

Use JSON:
[{"sid": <sid>, "cid": <cid>}]
"###;
}

pub fn determine_inglehart_welzel_map() {
    r###"
Family ties stay forever: Yes.

Regarding the above response of a person to a statement. Categorize the person with "none", "some" or "great" for each of the following scales and explain:
1. Traditional values
2. secular-rational values
3. Survival values
4. self-expression values
"###;
}

pub async fn determine_big_five_persona_trait(
    statement: &str,
) -> anyhow::Result<data::BigFivePersonaTrait> {
    let priming = ChatCompletionMessage {
        role: openai::chat::ChatCompletionMessageRole::System,
        content: r###"
You act as a tool to categorize public statements according to the big five personality traits.

I sometimes enjoy trying new foods and cuisines from different cultures.
JSON:
{"big-five-personality-trait": "openness-to-experience", "value": "medium", "notes": ""}

I talk to a lot of different people at parties.
JSON:
{"big-five-personality-trait": "extraversion", "value": "high", "notes": ""}

The way refugees behave in germany is outrageous and should be sanctioned!
JSON:
{"big-five-personality-trait": "agreeableness", "value": "low", "notes": "hateful"}

"###
        .to_string(),
        name: None,
    };
    let question = ChatCompletionMessage {
        role: openai::chat::ChatCompletionMessageRole::User,
        content: format!(
            r###"
{}
JSON:
"###,
            statement
        )
        .to_string(),
        name: None,
    };

    let raw_json_result = ChatCompletion::builder("gpt-3.5-turbo", vec![priming, question])
        .create()
        .await?
        .unwrap()
        .choices
        .first()
        .unwrap()
        .message
        .content
        .to_owned();
    println!("GPT raw result: {}", raw_json_result);
    let parsed_json: Value = serde_json::from_str(&raw_json_result)?;
    println!("GPT parsed result: {}", parsed_json);
    let axis = data::BigFivePersonaAxis::from_str(
        parsed_json["big-five-personality-trait"]
            .as_str()
            .ok_or(anyhow!("Unable to parse trait"))?,
    );
    let value = data::BigFivePersonaValue::from_str(
        parsed_json["value"]
            .as_str()
            .ok_or(anyhow!("Unable to parse trait"))?,
    );

    Ok(data::BigFivePersonaTrait { axis, value })
}

pub async fn run_dummy_prompt() {
    println!("Running GPT");
    let result = determine_big_five_persona_trait("I am upset about the political situation").await;

    println!("Got GPT result");
    match result {
        Ok(result) => println!("GPT OK: {:?}", result),
        Err(err) => println!("GPT Error: {:?}", err),
    };
}

pub async fn setup_openai() {
    set_key(env::var("OPENAI_KEY").unwrap());
}
