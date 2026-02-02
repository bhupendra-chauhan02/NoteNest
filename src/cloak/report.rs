use serde::{Deserialize, Serialize};

use crate::cloak::rules::PhiType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub file: String,
    pub counts: Vec<PhiCount>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiCount {
    pub phi_type: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateReport {
    pub files: Vec<FileReport>,
}

impl AggregateReport {
    pub fn empty() -> Self {
        Self { files: Vec::new() }
    }
}

pub fn count_for_phi(counts: &std::collections::HashMap<PhiType, usize>) -> Vec<PhiCount> {
    let mut output: Vec<PhiCount> = counts
        .iter()
        .map(|(phi, count)| PhiCount {
            phi_type: format!("{phi:?}").to_lowercase(),
            count: *count,
        })
        .collect();
    output.sort_by(|a, b| a.phi_type.cmp(&b.phi_type));
    output
}
