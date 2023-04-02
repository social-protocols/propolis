use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::structs::Statement;

use super::{api::{AiMessage, AiPrompt, PromptResponse}, multi_statement_classifier::MultiStatementPrompt};

/// A generic prompt yielding a single result
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
            env_info: r.env_info,
            prompt_info: r.prompt_info,
            content: (self.handler)(r.content),
            completion_tokens: r.completion_tokens,
            prompt_tokens: r.prompt_tokens,
            total_tokens: r.total_tokens,
        }
    }
}

/// Computes the big five personality traits for a statement
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Holds a weighting score
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Score {
    Weak,
    Strong,
    Unknown(String),
}

impl TryFrom<&str> for Score {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "w" => Ok(Score::Weak),
            "s" => Ok(Score::Strong),
            _ => Ok(Score::Unknown(value.into())),
        }
    }
}

/// A value with an optional score
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ScoredValue {
    pub value: String,
    pub score: Score,
}

pub fn is_valid_value(s: &str) -> bool {
    !vec!["none", "null", "-", ""].contains(&s.trim())
}

impl TryFrom<&str> for ScoredValue {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((value, score)) = value.rsplit_once(":") {
            Ok(ScoredValue {
                value: value.into(),
                score: score.try_into()?,
            })
        } else if is_valid_value(value) {
            Ok(ScoredValue {
                value: value.into(),
                score: Score::Unknown("".into()),
            })
        } else {
            Err(anyhow!("No value passed"))
        }
    }
}

/// Contains various meta information about a statement
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StatementMeta {
    Politics {
        tags: Vec<ScoredValue>,
        ideologies: Vec<ScoredValue>,
    },
    Personal {
        tags: Vec<ScoredValue>,
        bfp_traits: Vec<ScoredValue>,
    },
    Unparseable(String),
}

/// Container for several StatementMeta instances
#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct StatementMetaContainer {
    pub value: Vec<StatementMeta>,
}

impl IntoIterator for StatementMetaContainer {
    type Item = String;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.value
            .into_iter()
            .map(|s| serde_json::to_string(&s).unwrap())
            .collect::<Vec<String>>()
            .into_iter()
    }
}

impl TryFrom<String> for StatementMetaContainer {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        StatementMeta::from_lines(value.as_str())
    }
}

impl StatementMeta {
    /// Creates a container of StatementMeta from CSV data with "|" delimiter and no header
    pub fn from_lines(s: &str) -> anyhow::Result<StatementMetaContainer> {
        debug!("Deserializing CSV results:\n\n{}\n\n", s);
        // What the csv record looks like in data types
        type CsvRecord = (u64, String, String, String, String, String, String, String);

        let mut result: Vec<Self> = vec![];
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'|')
            .from_reader(s.as_bytes());
        for line in rdr.deserialize() {
            let record: CsvRecord = line?;

            let category = record.1.as_str();
            let tags = vec![
                record.5.as_str().try_into(),
                record.6.as_str().try_into(),
                record.7.as_str().try_into(),
            ]
            .into_iter()
            .flat_map(|i| i)
            .collect();
            result.push(match category {
                "politics" => Self::Politics {
                    tags,
                    ideologies: vec![
                        record.2.as_str().try_into(),
                        record.3.as_str().try_into(),
                        record.4.as_str().try_into(),
                    ]
                    .into_iter()
                    .flat_map(|i| i)
                    .collect(),
                },
                "personal" => Self::Personal {
                    tags,
                    bfp_traits: vec![
                        record.2.as_str().try_into(),
                        record.3.as_str().try_into(),
                        record.4.as_str().try_into(),
                    ]
                    .into_iter()
                    .flat_map(|i| i)
                    .collect(),
                },
                _ => Self::Unparseable(serde_json::to_string(&record).unwrap()),
            })
        }

        Ok(StatementMetaContainer { value: result })
    }

    /// Gives a prompt that computes various meta information on the passed statements
    pub fn prompt(stmts: &[Statement]) -> MultiStatementPrompt<StatementMetaContainer> {
        let mut stmts_s = String::from("");
        for s in stmts {
            stmts_s += format!("{}: {}", s.id, s.text).as_str();
        }
        MultiStatementPrompt {
            name: "statement_meta".into(),
            version: 8,
            handler: |s| {
                let s_without_header = s.trim().splitn(2, "\n").nth(1).unwrap_or("").to_string();
                let target_delim_count = 7;
                s_without_header
                    .split("\n")
                    // -- fixup delimiter count, since the ai does not reliably do that --
                    .map(|s| -> String {
                        let s = s.to_string();
                        let delim_count = s.match_indices("|").collect::<Vec<_>>().len();
                        if delim_count < target_delim_count {
                            s + "|".repeat(target_delim_count - delim_count).as_str().into()
                        } else if delim_count > target_delim_count {
                            s.strip_suffix("|".repeat(delim_count - target_delim_count).as_str())
                                .unwrap()
                                .into()
                        } else {
                            s
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    // -- finally try to return an R (result type) --
                    .try_into()
                    .expect("Unable to extract from string")
            },
            primer: vec![
                AiMessage::system(
                    "
You will be given multiple statements, each starting on their own line,
and your task is to determine whether the statement falls into the category
of politics or personal statements. In the case of it being a political category,
give which political ideologies (e.g., liberalism, conservatism, socialism)
each quote aligns with the most.
In the case of it being a personal category, give the big five personality traits instead.

In addition, also output up to three topic tags. The output should be a csv table with empty values as \"-\".
All cells should be followed by a strength score (w=weak, s=strong) after a \":\" delimiter.
If you are not sure, use \"-\"",
                ),
                AiMessage::user(
                    "
1. The global economy is at risk of recession due to the trade war and uncertainty it creates.
2. In clubs kann man hervorragend neue Freunde kennenlernen",
                ),
                AiMessage::assistant(
                    "
num|category|label1|label2|label3|tag1|tag2|tag3
1|politics|neoliberalism:s|conservatism:w|socialism:w|global economy:s|trade war:s|uncertainty:s
2|personal|extraversion:s|openness:w|agreeableness:s|clubs:s|friendship:s|socializing:w
",
                ),
                AiMessage::user(format!("{}", stmts_s).as_str()),
            ],
            stmts: stmts.to_vec(),
        }
    }
}

#[test]
fn test_statement_meta_from_lines() {
    let v = StatementMeta::from_lines(
        concat!(
            "1|politics|conservatism:s|nationalism:s|law and order:s|immigration:s|border security:w|protectionism:w\n",
        )
        ).unwrap();
    assert_eq!(v.value.len(), 1);
    let meta = &v.value[0];
    match meta {
        StatementMeta::Politics {
            tags,
            ideologies: _,
        } => {
            assert_eq!(tags[0].value, "immigration");
            assert_eq!(tags[0].score, Score::Strong);
        }
        _ => {
            assert!(false);
        }
    }
}
