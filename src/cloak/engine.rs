use std::collections::HashMap;

use regex::Regex;

use crate::cloak::config::CloakConfig;
use crate::cloak::report::{AggregateReport, FileReport, count_for_phi};
use crate::cloak::rules::{PhiRule, PhiType, default_rules, phi_label};
use crate::notenest::PlaceholderStyle;

#[derive(Debug, Clone)]
pub struct CloakResult {
    pub protected_text: String,
    pub report: FileReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualReviewFlag {
    MultipleIdentifiers,
    LongDigitSequence,
    ShortOrEmpty,
}

pub struct CloakEngine {
    config: CloakConfig,
    rules: Vec<PhiRule>,
    name_rule: Regex,
}

impl CloakEngine {
    pub fn new(config: CloakConfig) -> Self {
        let mut rules = default_rules();
        for custom in &config.custom_regex {
            if let Ok(regex) = Regex::new(&custom.pattern) {
                rules.push(PhiRule {
                    phi_type: PhiType::Other,
                    regex,
                    label: "custom",
                });
            }
        }

        let name_rule =
            Regex::new(r"(?i)\b(Dr\.?|Prof\.?|Herr|Frau)?\s*([A-Z][a-z]+\s+[A-Z][a-z]+)\b")
                .unwrap();

        Self {
            config,
            rules,
            name_rule,
        }
    }

    pub fn protect_text(&self, input: &str) -> CloakResult {
        let mut counts: HashMap<PhiType, usize> = HashMap::new();
        let mut flags: Vec<String> = Vec::new();
        let mut name_map: HashMap<String, String> = HashMap::new();
        let mut name_counter = 1;

        let style = map_style(self.config.placeholder_style.clone());
        let mut protected_text = input.to_string();

        protected_text = self
            .name_rule
            .replace_all(&protected_text, |caps: &regex::Captures| {
                let title = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let full = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let key = full.to_string();
                let entry = name_map.entry(key).or_insert_with(|| {
                    let label = if title.to_lowercase().starts_with("dr") {
                        format!("DOCTOR_{}", name_counter)
                    } else {
                        format!("NAME_{}", name_counter)
                    };
                    name_counter += 1;
                    label
                });
                *counts.entry(PhiType::Name).or_insert(0) += 1;
                let prefix = if title.is_empty() {
                    String::new()
                } else {
                    format!("{} ", title)
                };
                format!("{}{}", prefix, entry)
            })
            .into_owned();

        for rule in &self.rules {
            let phi_type = rule.phi_type;
            protected_text = rule
                .regex
                .replace_all(&protected_text, |_: &regex::Captures| {
                    *counts.entry(phi_type).or_insert(0) += 1;
                    placeholder_for(phi_type, style)
                })
                .into_owned();
        }

        let mut phi_line_counts = 0;
        for line in protected_text.lines() {
            let mut line_hits = 0;
            for rule in &self.rules {
                if rule.regex.is_match(line) {
                    line_hits += 1;
                }
            }
            if line_hits >= 2 {
                phi_line_counts += 1;
            }
        }
        if phi_line_counts > 0 {
            flags.push("multiple_identifiers".to_string());
        }

        if input.len() < self.config.thresholds.min_note_length {
            flags.push("short_note".to_string());
        }

        if Regex::new(&format!(
            r"\b\d{{{},{}}}\b",
            self.config.thresholds.digit_length,
            self.config.thresholds.digit_length + 6
        ))
        .unwrap()
        .is_match(input)
        {
            flags.push("long_digit_sequence".to_string());
        }

        CloakResult {
            protected_text,
            report: FileReport {
                file: "inline".into(),
                counts: count_for_phi(&counts),
                flags,
            },
        }
    }

    pub fn protect_file(&self, file: &str, input: &str) -> CloakResult {
        let mut result = self.protect_text(input);
        result.report.file = file.to_string();
        result
    }

    pub fn aggregate_report(&self, results: Vec<FileReport>) -> AggregateReport {
        AggregateReport { files: results }
    }
}

fn map_style(style: crate::cloak::config::PlaceholderStyleConfig) -> PlaceholderStyle {
    match style {
        crate::cloak::config::PlaceholderStyleConfig::Protected => PlaceholderStyle::Protected,
        crate::cloak::config::PlaceholderStyleConfig::Masked => PlaceholderStyle::Masked,
        crate::cloak::config::PlaceholderStyleConfig::Hidden => PlaceholderStyle::Hidden,
        crate::cloak::config::PlaceholderStyleConfig::Removed => PlaceholderStyle::Removed,
        crate::cloak::config::PlaceholderStyleConfig::Angle => PlaceholderStyle::Angle,
    }
}

fn placeholder_for(phi: PhiType, style: PlaceholderStyle) -> String {
    let kind = match phi {
        PhiType::Email => "EMAIL",
        PhiType::Phone => "PHONE",
        PhiType::Url => "URL",
        PhiType::Mrn => "MRN",
        PhiType::Insurance => "INSURANCE",
        PhiType::Date => "DATE",
        PhiType::Address => "ADDRESS",
        PhiType::PostalCode => "POSTAL",
        PhiType::Id => "ID",
        PhiType::Name => "NAME",
        PhiType::Other => "OTHER",
    };

    match style {
        PlaceholderStyle::Protected => format!("[{}_PROTECTED]", kind),
        PlaceholderStyle::Masked => format!("[{}_MASKED]", kind),
        PlaceholderStyle::Hidden => format!("[{}_HIDDEN]", kind),
        PlaceholderStyle::Removed => format!("[{}_REMOVED]", kind),
        PlaceholderStyle::Angle => format!("<{}>", kind),
    }
}

pub fn rule_summary(counts: &HashMap<PhiType, usize>) -> Vec<(String, usize)> {
    let mut items: Vec<(String, usize)> = counts
        .iter()
        .map(|(phi, count)| (phi_label(*phi).to_string(), *count))
        .collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    items
}
