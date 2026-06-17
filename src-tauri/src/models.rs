use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub original: String,
    pub correction: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub position: usize,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub original: String,
    pub correction: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAiSuggestion {
    pub original: String,
    #[serde(alias = "korrektur")]
    pub correction: String,
    #[serde(alias = "typ")]
    pub kind: String,
    #[serde(default, alias = "erklaerung")]
    pub explanation: String,
}
