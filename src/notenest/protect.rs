use super::redaction::redact_note;
use super::types::{PlaceholderStyle, RedactionResult};

pub fn protect_note(input: &str, style: PlaceholderStyle) -> RedactionResult {
    redact_note(input, style)
}
