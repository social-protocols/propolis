use anyhow::anyhow;
use openai::chat::{ChatCompletion, ChatCompletionMessage};
use openai::set_key;
use serde_json::Value;
use std::env;

use crate::error::Error;

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

#[derive(Debug)]
pub enum BigFivePersonaAxis {
    OpennessToExperience,
    Conscientiousness,
    Extraversion,
    Agreeableness,
    Neuroticism,
    Unknown(String),
}

impl BigFivePersonaAxis {
    pub fn from_str(s: &str) -> BigFivePersonaAxis {
        match s {
            "openness-to-experience" => BigFivePersonaAxis::OpennessToExperience,
            "conscientiousness" => BigFivePersonaAxis::Conscientiousness,
            "extraversion" => BigFivePersonaAxis::Extraversion,
            "agreeableness" => BigFivePersonaAxis::Agreeableness,
            "neuroticism" => BigFivePersonaAxis::Neuroticism,
            s => BigFivePersonaAxis::Unknown(s.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum BigFivePersonaValue {
    Low,
    Medium,
    High,
    Unknown(String),
}

impl BigFivePersonaValue {
    pub fn from_str(s: &str) -> BigFivePersonaValue {
        match s {
            "high" => BigFivePersonaValue::High,
            "medium" => BigFivePersonaValue::Medium,
            "low" => BigFivePersonaValue::Low,
            s => BigFivePersonaValue::Unknown(s.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct BigFivePersonaTrait {
    pub axis: BigFivePersonaAxis,
    pub value: BigFivePersonaValue,
}

pub async fn determine_big_five_persona_trait(
    statement: &str,
) -> anyhow::Result<BigFivePersonaTrait> {
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
    let axis = BigFivePersonaAxis::from_str(
        parsed_json["big-five-personality-trait"]
            .as_str()
            .ok_or(anyhow!("Unable to parse trait"))?,
    );
    let value = BigFivePersonaValue::from_str(
        parsed_json["value"]
            .as_str()
            .ok_or(anyhow!("Unable to parse trait"))?,
    );

    Ok(BigFivePersonaTrait { axis, value })
}

pub async fn setup_openai() {
    set_key(env::var("OPENAI_KEY").unwrap());

    println!("Running GPT");
    let result =
        determine_big_five_persona_trait("I am upset about the political situation").await;

    println!("Got GPT result");
    match result {
        Ok(result) => println!("GPT OK: {:?}", result),
        Err(err) => println!("GPT Error: {:?}", err),
    };
}
