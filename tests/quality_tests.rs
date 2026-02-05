use regex::Regex;

use notenest::notenest::{ClinicianMode, render_text_output_with_mode};
use notenest::{PlaceholderStyle, process_note, summarize_note};

#[test]
fn phone_placeholders_have_no_trailing_digits() {
    let input = "wife Mary 0176-12345678 called +49 152 98765432";
    let result = process_note(input, PlaceholderStyle::Protected);
    let output = render_text_output_with_mode(&result, ClinicianMode::Soap);
    let re = Regex::new(r"\[PHONE_PROTECTED\]\d").unwrap();
    assert!(!re.is_match(&output));
}

#[test]
fn placeholders_never_followed_by_digits() {
    let input = "call +49 176 12345678 email test@example.com MRN 883920 DOB 12/03/1982 addr 12 Hauptstrasse 80331 Muenchen";
    let result = process_note(input, PlaceholderStyle::Protected);
    let output = render_text_output_with_mode(&result, ClinicianMode::Soap);
    let re = Regex::new(r"\[(EMAIL|PHONE|ID|DOB|ADDRESS|NAME)_[A-Z]+\]\d").unwrap();
    assert!(!re.is_match(&output));
}

#[test]
fn nkda_maps_to_allergies() {
    let input = "pmh htn. nkda.";
    let result = process_note(input, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    assert!(
        summary
            .allergies
            .iter()
            .any(|a| a.to_lowercase().contains("no known drug allergies"))
    );
}

#[test]
fn clinician_mode_toggle_respected() {
    let input = "abdo pain x3d";
    let result = process_note(input, PlaceholderStyle::Protected);
    let soap = render_text_output_with_mode(&result, ClinicianMode::Soap);
    let five_cs = render_text_output_with_mode(&result, ClinicianMode::FiveCs);

    assert!(!soap.contains("5C's"));
    assert!(!five_cs.contains("SOAP - S"));
}

#[test]
fn vitals_and_tests_extracted() {
    let input = "BP168/96 HR108 T37.2 trop 0.08 ng/mL";
    let result = process_note(input, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    assert!(summary.vitals.iter().any(|v| v.contains("BP")));
    assert!(
        summary
            .tests
            .iter()
            .any(|t| t.to_lowercase().contains("trop"))
    );
}

#[test]
fn complaint_is_short_phrase() {
    let input = "ER note: chest tightness x2d, worse stairs + SOB.";
    let result = process_note(input, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    let complaint = summary.chief_concern.join(" ");
    let word_count = complaint.split_whitespace().count();
    assert!(word_count <= 12);
    assert!(!complaint.to_lowercase().contains("er note"));
}

#[test]
fn plan_does_not_include_address() {
    let input = "plan: send ED repeat trop 3h start ASA follow-up cardio.\naddr 12 Hauptstrasse 80331 Muenchen.";
    let result = process_note(input, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    let plan = summary.plan.join(" ").to_lowercase();
    assert!(!plan.contains("address"));
    assert!(!plan.contains("[address_protected]"));
    assert!(!plan.contains("hauptstrasse"));
}

#[test]
fn patient_view_has_main_concern() {
    let input = "pt walked in can't breathe well since monday worse stairs.";
    let result = process_note(input, PlaceholderStyle::Protected);
    let output = render_text_output_with_mode(&result, ClinicianMode::Soap);
    assert!(output.contains("What you came in with"));
    assert!(output.contains("Main concern:"));
}

#[test]
fn extracts_plan_from_plan_gt_inline_semicolons() {
    let input = "PLAN> send ED; repeat trop 3h; start ASA; send ED";
    let result = process_note(input, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    assert!(
        summary
            .plan
            .iter()
            .any(|p| p.to_lowercase().contains("send ed"))
    );
    assert!(
        summary
            .plan
            .iter()
            .any(|p| p.to_lowercase().contains("repeat trop"))
    );
    assert!(
        summary
            .plan
            .iter()
            .any(|p| p.to_lowercase().contains("start asa"))
    );
}

#[test]
fn meds_cleaning_splits_drug_and_dose() {
    let input = "pmh?? HTN DM2 meds: metformin500bid + ramipril5mg od NKDA";
    let result = process_note(input, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    let meds = summary.meds.join(" ").to_lowercase();
    assert!(meds.contains("metformin 500 bid"));
    assert!(meds.contains("ramipril 5 mg od") || meds.contains("ramipril 5 od"));
    assert!(!meds.contains("nkda"));
}

#[test]
fn address_placeholder_increments_count() {
    let input = "addr: 12 Hauptstrasse 80331 Muenchen";
    let result = process_note(input, PlaceholderStyle::Protected);
    assert!(result.coverage.protected_counts.addresses > 0);
}
