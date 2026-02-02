use notenest::notenest::render_text_output;
use notenest::{PlaceholderStyle, process_note, summarize_note};

const NOTE1: &str = r"ER note (messy): JOHN O?? 43M chest tightness x2d, worse stairs + SOB. wife Mary 0176-12345678 called.
email john.osmith@gmail.com MRN 883920 DOB 12/03/1982 addr 12 Hauptstrasse 80331 Muenchen.
pmh HTN/DM2; meds metformin 500 bid + ramipril 5mg od; nkda.
vitals BP168/96 HR108 T37.2. ecg ?st-depr. trop 0.08 ng/mL.
plan: send ED; repeat trop 3h; ASA; consider heparin; f/u cardio.";

const NOTE2: &str = r#"walk-in messy note: "abdo pain??" started last monday; worse after meals.
stress @ work; sleeps 3-4h. denies vomiting; some diarrhea.
contact sara.khan@web.de +49 152 98765432 ID# AOK-1199-22.
meds unsure "ibu sometimes". allergy: penicillin rash."#;

#[test]
fn messy_note1_extracts_fields() {
    let result = process_note(NOTE1, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    assert!(!summary.chief_concern.is_empty());
    assert!(!summary.duration.is_empty());
    assert!(!summary.meds.is_empty());
    assert!(!summary.plan.is_empty());
    assert!(!summary.vitals.is_empty());
    assert!(!summary.tests.is_empty());

    let output = render_text_output(&result);
    assert!(output.contains("[EMAIL_PROTECTED]"));
    assert!(output.contains("[PHONE_PROTECTED]"));
    assert!(output.contains("[ID_PROTECTED]") || output.contains("[MRN_PROTECTED]"));
    assert!(output.contains("[DOB_PROTECTED]"));
    assert!(output.contains("[ADDRESS_PROTECTED]"));
}

#[test]
fn messy_note2_extracts_fields() {
    let result = process_note(NOTE2, PlaceholderStyle::Protected);
    let summary = summarize_note(&result.protected_text);
    assert!(!summary.chief_concern.is_empty());
    assert!(!summary.duration.is_empty());
    assert!(!summary.meds.is_empty());
    assert!(!summary.allergies.is_empty());

    let output = render_text_output(&result);
    assert!(output.contains("[EMAIL_PROTECTED]"));
    assert!(output.contains("[PHONE_PROTECTED]"));
    assert!(output.contains("[ID_PROTECTED]"));
}
