use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloakConfig {
    #[serde(default)]
    pub placeholder_style: PlaceholderStyleConfig,
    #[serde(default)]
    pub thresholds: Thresholds,
    #[serde(default)]
    pub custom_regex: Vec<CustomRegex>,
    #[serde(default)]
    pub dictionaries: Dictionaries,
}

impl Default for CloakConfig {
    fn default() -> Self {
        Self {
            placeholder_style: PlaceholderStyleConfig::Protected,
            thresholds: Thresholds::default(),
            custom_regex: Vec::new(),
            dictionaries: Dictionaries::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlaceholderStyleConfig {
    #[default]
    Protected,
    Masked,
    Hidden,
    Removed,
    Angle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    #[serde(default = "default_digit_length")]
    pub digit_length: usize,
    #[serde(default = "default_min_note_length")]
    pub min_note_length: usize,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            digit_length: default_digit_length(),
            min_note_length: default_min_note_length(),
        }
    }
}

fn default_digit_length() -> usize {
    6
}

fn default_min_note_length() -> usize {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRegex {
    pub name: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dictionaries {
    #[serde(default)]
    pub titles: Vec<String>,
    #[serde(default)]
    pub first_names: Vec<String>,
    #[serde(default)]
    pub last_names: Vec<String>,
}

impl Default for Dictionaries {
    fn default() -> Self {
        Self {
            titles: vec!["Dr".into(), "Prof".into(), "Herr".into(), "Frau".into()],
            first_names: Vec::new(),
            last_names: Vec::new(),
        }
    }
}
