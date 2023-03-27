use openai::chat::{ChatCompletion, ChatCompletionMessage};
use openai::set_key;
use std::env;

pub async fn setup_openai() {
    set_key(env::var("OPENAI_KEY").unwrap());
    let message = ChatCompletionMessage {
        role: openai::chat::ChatCompletionMessageRole::User,
        content: "Say hello world".to_string(),
        name: None,
    };
    let completion = ChatCompletion::builder("gpt-3.5-turbo", vec![message])
        .create()
        .await
        .unwrap()
        .unwrap()
        .choices
        .first()
        .unwrap()
        .message
        .to_owned();
    format!("Result from gpt: {}", completion.content.trim());
}
