#[derive(Debug)]
#[allow(unused_imports, dead_code)]
pub enum BigFivePersonaAxis {
    OpennessToExperience,
    Conscientiousness,
    Extraversion,
    Agreeableness,
    Neuroticism,
    Unknown(String),
}

impl BigFivePersonaAxis {
    #[allow(dead_code)]
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
#[allow(unused_imports, dead_code)]
pub enum BigFivePersonaValue {
    Low,
    Medium,
    High,
    Unknown(String),
}

impl BigFivePersonaValue {
    #[allow(dead_code)]
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
#[allow(unused_imports)]
pub struct BigFivePersonaTrait {
    pub axis: BigFivePersonaAxis,
    pub value: BigFivePersonaValue,
}

