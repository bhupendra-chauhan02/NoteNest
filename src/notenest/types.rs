use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaceholderStyle {
    Protected,
    Masked,
    Hidden,
    Removed,
    Angle,
}

impl PlaceholderStyle {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(input)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Protected => "protected",
            Self::Masked => "masked",
            Self::Hidden => "hidden",
            Self::Removed => "removed",
            Self::Angle => "angle",
        }
    }
}

impl std::str::FromStr for PlaceholderStyle {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "protected" => Ok(Self::Protected),
            "masked" => Ok(Self::Masked),
            "hidden" => Ok(Self::Hidden),
            "removed" => Ok(Self::Removed),
            "angle" => Ok(Self::Angle),
            _ => Err(format!(
                "invalid placeholder style: {} (use protected, masked, hidden, removed, angle)",
                input
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RedactionCounts {
    pub names: usize,
    pub phones: usize,
    pub emails: usize,
    pub dobs: usize,
    pub ids: usize,
    pub addresses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionResult {
    pub redacted_text: String,
    pub counts: RedactionCounts,
    pub style: PlaceholderStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SummaryResult {
    pub chief_concern: Vec<String>,
    pub duration: Vec<String>,
    pub symptoms: Vec<String>,
    pub pmh: Vec<String>,
    pub meds: Vec<String>,
    pub allergies: Vec<String>,
    pub vitals: Vec<String>,
    pub tests: Vec<String>,
    pub key_findings: Vec<String>,
    pub assessment: Vec<String>,
    pub plan: Vec<String>,
    pub context: Vec<String>,
    pub concerns: Vec<String>,
    pub coping: Vec<String>,
    pub key_results: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PatientFound {
    pub symptoms: Vec<String>,
    pub negatives: Vec<String>,
    pub medications: Vec<String>,
    pub allergies: Vec<String>,
    pub tests_results: Vec<String>,
    pub vitals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientView {
    pub main_concern: String,
    pub onset_duration: String,
    pub triggers: Vec<String>,
    pub what_it_could_mean: String,
    pub what_we_found: PatientFound,
    pub next_steps: Vec<String>,
    pub questions_to_ask: Vec<String>,
    pub urgent_red_flags: Vec<String>,
    pub disclaimer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClinicianSoap {
    #[serde(rename = "S")]
    pub s: Vec<String>,
    #[serde(rename = "O")]
    pub o: Vec<String>,
    #[serde(rename = "A")]
    pub a: Vec<String>,
    #[serde(rename = "P")]
    pub p: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clinician5Cs {
    pub chief_complaint: String,
    pub course: Vec<String>,
    pub context: Vec<String>,
    pub concerns: Vec<String>,
    pub coping: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub fields_found: usize,
    pub fields_missing: Vec<String>,
    pub protected_counts: RedactionCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteNestOutputs {
    pub protected_text: String,
    pub placeholder_style: PlaceholderStyle,
    pub patient_view: PatientView,
    pub clinician_soap: ClinicianSoap,
    pub clinician_5cs: Clinician5Cs,
    pub coverage: CoverageReport,
}

pub type ProcessResult = NoteNestOutputs;
