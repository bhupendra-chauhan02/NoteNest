use regex::Regex;

use notenest::{PlaceholderStyle, process_note};

const MESSY_NOTE: &str = "12:07 triage note — pt 'cant breathe' chest tight?? since Mon worse w stairs sweaty. denies fever. wife +49 176 12345678 email john.osmith@gmail.com MRN 883920 DOB 12/03/1982 addr 12 Hauptstrasse 80331 München. pmh HTN DM2 meds metformin 500 bid ramipril 5mg od NKDA BP168/96 HR108 T37.2 ECG ?st depr trop 0.08 ng/mL plan: send ED repeat trop 3h start ASA consider heparin cardiology f/u. random junk: !!! copied template text ………";

#[test]
fn patient_view_strips_timestamps_and_junk() {
    let result = process_note(MESSY_NOTE, PlaceholderStyle::Protected);
    let output = notenest::notenest::render_text_output_with_mode(
        &result,
        notenest::notenest::ClinicianMode::Soap,
    );
    assert!(!output.contains("12:07"));
    assert!(!output.to_lowercase().contains("triage note"));
    assert!(!output.to_lowercase().contains("random junk"));
    assert!(!output.to_lowercase().contains("copied template"));
}

#[test]
fn main_concern_is_short_phrase() {
    let result = process_note(MESSY_NOTE, PlaceholderStyle::Protected);
    let words = result.patient_view.main_concern.split_whitespace().count();
    assert!(words <= 8);
}

#[test]
fn plan_bullets_do_not_include_phi_placeholders() {
    let result = process_note(MESSY_NOTE, PlaceholderStyle::Protected);
    for item in &result.clinician_soap.p {
        let lower = item.to_lowercase();
        assert!(!lower.contains("address"));
        assert!(!lower.contains("[address_protected]"));
        assert!(!lower.contains("[id_protected]"));
        assert!(!lower.contains("[dob_protected]"));
    }
}

#[test]
fn vitals_and_negatives_extracted() {
    let result = process_note(MESSY_NOTE, PlaceholderStyle::Protected);
    let vitals = &result.patient_view.what_we_found.vitals;
    assert!(vitals.iter().any(|v| v.contains("BP")));
    assert!(vitals.iter().any(|v| v.contains("168/96")));
    let negatives = &result.patient_view.what_we_found.negatives;
    assert!(
        negatives
            .iter()
            .any(|n| n.to_lowercase().contains("denies"))
    );
}

#[test]
fn protected_note_has_clean_placeholders() {
    let result = process_note(MESSY_NOTE, PlaceholderStyle::Protected);
    let re_plus = Regex::new(r"\+\s*\[PHONE_PROTECTED\]").unwrap();
    let re_trailing = Regex::new(r"\[PHONE_PROTECTED\]\d").unwrap();
    assert!(!re_plus.is_match(&result.protected_text));
    assert!(!re_trailing.is_match(&result.protected_text));
}
