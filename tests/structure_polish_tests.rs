use notenest::{PlaceholderStyle, process_note};

const FIXTURE: &str = "12:07 triage note — pt 'cant breathe' chest tight?? since Mon worse w stairs sweaty. denies fever. contact +49 176 12345678 email john.osmith@gmail.com MRN 883920 DOB 12/03/1982 addr 12 Hauptstrasse 80331 München. pmh HTN DM2 meds metformin 500 bid ramipril 5mg od NKDA BP168/96 HR108 T37.2 SpO2 92 ECG ?st depr trop 0.08 ng/mL plan: > send ED; repeat trop 3h; start ASA; consider heparin; send ED.";

#[test]
fn tests_results_excludes_phi_placeholders() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let tests = result.patient_view.what_we_found.tests_results.join(" ");
    assert!(!tests.contains("[EMAIL_PROTECTED]"));
    assert!(!tests.contains("[PHONE_PROTECTED]"));
    assert!(!tests.contains("[ID_PROTECTED]"));
    assert!(!tests.contains("[DOB_PROTECTED]"));
    assert!(!tests.contains("[ADDRESS_PROTECTED]"));
}

#[test]
fn tests_results_exclude_vitals_line() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let tests = result
        .patient_view
        .what_we_found
        .tests_results
        .join(" ")
        .to_lowercase();
    assert!(!tests.contains("vitals"));
    assert!(!tests.contains("bp168/96"));
    assert!(!tests.contains("bp 168/96"));
    assert!(!tests.contains("spo2"));
}

#[test]
fn plan_bullets_clean_and_deduped() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let plan = result.clinician_soap.p;
    let send_count = plan
        .iter()
        .filter(|p| p.to_lowercase() == "send ed")
        .count();
    assert_eq!(send_count, 1);
    assert!(!plan.iter().any(|p| p.trim_start().starts_with('>')));
    assert!(!plan.iter().any(|p| p.contains("[ADDRESS_PROTECTED]")));
    assert!(!plan.iter().any(|p| p.contains("[EMAIL_PROTECTED]")));
}

#[test]
fn spo2_formats_cleanly() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let vitals = result.patient_view.what_we_found.vitals.join(" ");
    assert!(vitals.contains("SpO2 92%") || vitals.contains("SpO2: 92%"));
}

#[test]
fn no_not_found_in_end_user_views() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let output = notenest::notenest::render_text_output_with_mode(
        &result,
        notenest::notenest::ClinicianMode::Soap,
    );
    assert!(!output.contains("Not found"));
}

#[test]
fn spo2_does_not_duplicate_raw_tokens() {
    let input = "vitals?? spo2 92 bp168/96 hr108";
    let result = process_note(input, PlaceholderStyle::Protected);
    let vitals = result
        .patient_view
        .what_we_found
        .vitals
        .join(" ")
        .to_lowercase();
    assert!(vitals.contains("spo2 92%") || vitals.contains("spo2: 92%"));
    assert!(!vitals.contains("spo 2 92"));
}

#[test]
fn medications_exclude_vitals_and_tests() {
    let input = "meds: metformin500bid + ramipril5mg od vitals BP168/96 HR108 SpO2 92 trop 0.08";
    let result = process_note(input, PlaceholderStyle::Protected);
    let meds = result
        .patient_view
        .what_we_found
        .medications
        .join(" ")
        .to_lowercase();
    assert!(meds.contains("metformin 500 bid"));
    assert!(meds.contains("ramipril 5 mg od") || meds.contains("ramipril 5 od"));
    assert!(!meds.contains("bp"));
    assert!(!meds.contains("hr"));
    assert!(!meds.contains("spo2"));
    assert!(!meds.contains("trop"));
    assert!(!meds.contains("0.08"));
}

#[test]
fn symptom_normalizes_shortness_of_breath() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let symptoms = result.patient_view.what_we_found.symptoms.join(" ");
    assert!(symptoms.contains("Shortness of breath"));
}

#[test]
fn protected_note_removes_junk_footer() {
    let result = process_note(FIXTURE, PlaceholderStyle::Protected);
    let lower = result.protected_text.to_lowercase();
    assert!(!lower.contains("do not share"));
    assert!(!lower.contains("random footer"));
}
