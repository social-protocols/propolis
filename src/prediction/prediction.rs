use super::{
    api::{AiEnv, AiMessage},
    prompts::GenericPrompt,
};

/// Computes the big five personality traits for a statement
pub async fn bfp<E: AiEnv>(s: &String, env: E) -> anyhow::Result<String> {
    let prompt = GenericPrompt {
        name: "BFP".to_string(),
        version: 1,
        handler: |s| s,
        primer: vec![
            AiMessage::system(concat!(
                "You act as a tool to categorize public ",
                "statements according to the big five personality traits.\n"
            )),
            AiMessage::user(concat!(
                "I sometimes enjoy trying new foods and cuisines from different cultures.\n",
                "JSON:\n"
            )),
            AiMessage::assistant(concat!(
                r###"{"big-five-personality-trait": "openness-to-experience", "value": "medium", "notes": ""}\n"###
            )),
            AiMessage::user(concat!(
                "I talk to a lot of different people at parties.\n",
                "JSON:\n"
            )),
            AiMessage::assistant(concat!(
                r###"{"big-five-personality-trait": "extraversion", "value": "high", "notes": ""}"###
            )),
            AiMessage::user(concat!(
                "The way refugees behave in germany is outrageous and should be sanctioned!\n",
                "JSON:\n"
            )),
            AiMessage::assistant(concat!(
                r###"{"big-five-personality-trait": "agreeableness", "value": "low", "notes": "hateful"}"###
            )),
            // the actual prediction
            AiMessage::user(format!("{}\nJSON:\n", s).as_str()),
        ],
    };

    Ok(env.send_prompt(&prompt).await?)
}
