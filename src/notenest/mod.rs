pub mod extract;
pub mod protect;
pub mod redaction;
pub mod render;
pub mod summary;
pub mod types;
pub mod util;

pub use protect::protect_note;
pub use redaction::{placeholder, redact_note};
pub use render::{
    ClinicianMode, render_clinician_view, render_coverage, render_patient_view, render_text_output,
    render_text_output_with_mode,
};
pub use summary::{build_clinician_5cs, build_clinician_soap, build_patient_view, summarize_note};
pub use types::{
    Clinician5Cs, ClinicianSoap, CoverageReport, NoteNestOutputs, PatientFound, PatientView,
    PlaceholderStyle, ProcessResult, RedactionCounts, RedactionResult, SummaryResult,
};

pub fn process_note(input: &str, style: PlaceholderStyle) -> ProcessResult {
    let normalized = util::normalize_text(input);
    let redaction = redact_note(&normalized, style);
    let summary = summarize_note(&redaction.redacted_text);
    let patient_view = build_patient_view(&summary);
    let clinician_soap = build_clinician_soap(&summary);
    let clinician_5cs = build_clinician_5cs(&summary);
    let coverage = render::build_coverage_report(&summary, &redaction.counts);

    ProcessResult {
        protected_text: redaction.redacted_text,
        placeholder_style: redaction.style,
        patient_view,
        clinician_soap,
        clinician_5cs,
        coverage,
    }
}
