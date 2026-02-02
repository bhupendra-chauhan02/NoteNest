use notenest::{PlaceholderStyle, redact_note};

#[test]
fn golden_note_uses_placeholder_tokens() {
    let note = "Name: John Doe\nDOB: 02/14/1978\nEmail: john@example.com\nPhone: (555) 123-4567\nMRN: 123456\nAddress: 742 Evergreen Terrace\n";
    let result = redact_note(note, PlaceholderStyle::Protected);

    assert!(result.redacted_text.contains("[NAME_PROTECTED]"));
    assert!(
        result.redacted_text.contains("[DOB_PROTECTED]"),
        "{}",
        result.redacted_text
    );
    assert!(result.redacted_text.contains("[EMAIL_PROTECTED]"));
    assert!(result.redacted_text.contains("[PHONE_PROTECTED]"));
    assert!(result.redacted_text.contains("[ID_PROTECTED]"));
    assert!(result.redacted_text.contains("[ADDRESS_PROTECTED]"));
    assert!(!result.redacted_text.contains("REDACTED"));
}
