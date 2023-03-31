use std::borrow::Borrow;

use function_name::named;
use qrcode::render::string::Element;

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

/// Tries to determine a statements ideology
pub fn statement_ideology(s: &Statement) -> GenericPrompt {
    GenericPrompt {
        name: "statement_ideology".to_string(),
        version: 3,
        handler: |s| s,
        primer: vec![
            AiMessage::system("Determine a statements ideology in single words."),
            AiMessage::user("More money must be invested."),
            AiMessage::assistant("capitalist"),
            AiMessage::user("Nature must be protected on a global scale."),
            AiMessage::assistant("environmentalist,globalist"),
            // the actual prediction
            AiMessage::user(s.text.as_str()),
        ],
    }
}

/// Given multiple statements, predict: category (political / personal),
/// political ideology or bfp traits and tags
#[named]
pub fn multi_statement_predictor<S: Borrow<Statement>>(stmts: &[S]) -> GenericPrompt {
    let mut stmts_s = String::from("");
    for s in stmts {
        stmts_s += format!("{}: {}", s.borrow().id, s.borrow().text).as_str();
    }
    GenericPrompt {
        name: function_name!().into(),
        version: 7,
        handler: |s| s,
        primer: vec![
            AiMessage::system("
You will be given multiple statements, each starting on their own line,
and your task is to determine whether the statement falls into the category
of politics or personal statements. In the case of it being a political category,
give which political ideologies (e.g., liberalism, conservatism, socialism)
each quote aligns with the most.
In the case of it being a personal category, give the big five personality traits instead.

In addition, also output up to three topic tags. The output should be a csv table.
All cells should be followed by a strength score (w=weak, s=strong) after a \":\" delimiter.
"),
            AiMessage::user("
1. The global economy is at risk of recession due to the trade war and uncertainty it creates.
2. In clubs kann man hervorragend neue Freunde kennenlernen
"),
            AiMessage::assistant("
num|category|label1|label2|label3|tag1|tag2|tag3
1|politics|neoliberalism:s|conservatism:w|socialism:w|global economy:s|trade war:s|uncertainty:s
2|personal|extraversion:s|openness:w|agreeableness:s|clubs:s|friendship:s|socializing:w
"),
            AiMessage::user(format!("{}", stmts_s).as_str()),
        ],
    }
}
