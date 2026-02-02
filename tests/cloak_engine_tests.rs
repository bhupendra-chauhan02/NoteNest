use notenest::cloak::{CloakConfig, CloakEngine};

#[test]
fn detects_email_and_phone() {
    let engine = CloakEngine::new(CloakConfig::default());
    let input = "Email jane@example.com phone 555-123-4567";
    let result = engine.protect_text(input);
    assert!(result.protected_text.contains("[EMAIL_PROTECTED]"));
    assert!(result.protected_text.contains("[PHONE_PROTECTED]"));
}

#[test]
fn pseudonymizes_names_consistently() {
    let engine = CloakEngine::new(CloakConfig::default());
    let input = "Dr Smith met Dr Smith. Dr Jones followed.";
    let result = engine.protect_text(input);
    let first = result.protected_text.matches("DOCTOR_").count();
    assert!(first >= 2);
}
