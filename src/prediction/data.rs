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

