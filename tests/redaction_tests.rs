use notenest::{PlaceholderStyle, redact_note};

#[test]
fn protected_replaces_email() {
    let input = "Contact: jane.doe@example.com";
    let result = redact_note(input, PlaceholderStyle::Protected);
    assert!(result.redacted_text.contains("[EMAIL_PROTECTED]"));
    assert!(!result.redacted_text.contains("REDACTED"));
}

#[test]
fn masked_replaces_phone() {
    let input = "Call 555-123-4567 for details";
    let result = redact_note(input, PlaceholderStyle::Masked);
    assert!(result.redacted_text.contains("[PHONE_MASKED]"));
}

#[test]
fn angle_style_uses_angle_tokens() {
    let input = "DOB: 01/02/1980";
    let result = redact_note(input, PlaceholderStyle::Angle);
    assert!(result.redacted_text.contains("<DOB>"));
}
