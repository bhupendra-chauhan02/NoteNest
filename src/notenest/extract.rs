use super::summary::{
    build_clinician_5cs, build_clinician_soap, build_patient_view, summarize_note,
};
use super::types::{Clinician5Cs, ClinicianSoap, PatientView, SummaryResult};

pub fn extract_summary(input: &str) -> SummaryResult {
    summarize_note(input)
}

pub fn build_patient(summary: &SummaryResult) -> PatientView {
    build_patient_view(summary)
}

pub fn build_soap(summary: &SummaryResult) -> ClinicianSoap {
    build_clinician_soap(summary)
}

pub fn build_5cs(summary: &SummaryResult) -> Clinician5Cs {
    build_clinician_5cs(summary)
}
