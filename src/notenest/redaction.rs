use once_cell::sync::Lazy;
use regex::Regex;

use super::types::{PlaceholderStyle, RedactionCounts, RedactionResult};

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}").unwrap());

static PHONE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:\+?\d[\d\s().-]{6,}\d)\b").unwrap());

static DOB_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?P<label>DOB|Date of Birth)[ \t]*[:\-]?[ \t]*(\d{1,2}[\/\-]\d{1,2}[\/\-]\d{2,4}|\d{4}[\/\-]\d{1,2}[\/\-]\d{1,2})",
    )
    .unwrap()
});

static NAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?P<label>Name|Patient Name|Patient|Pt)[ \t]*:[ \t]*[A-Z][a-z]+(?:[ \t]+[A-Z][a-z]+){1,2}",
    )
    .unwrap()
});

static ADDRESS_LABEL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(Address|Addr)\s*[:\-]?\s*.*").unwrap());

static ADDRESS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\b\d{1,5}\s+[A-Za-z0-9.'-]+(?:\s+[A-Za-z0-9.'-]+){0,4}\s+(Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr|Court|Ct|Way|Place|Pl|Strasse|Str)\b",
    )
    .unwrap()
});

static ID_LABEL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?P<label>ID|MRN|Record|Account)[ \t]*[:#]?[ \t]*[A-Z0-9-]{4,}\b").unwrap()
});

static ID_GENERIC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{6,}\b").unwrap());

pub fn placeholder(kind: &str, style: PlaceholderStyle) -> String {
    match style {
        PlaceholderStyle::Protected => format!("[{}_PROTECTED]", kind),
        PlaceholderStyle::Masked => format!("[{}_MASKED]", kind),
        PlaceholderStyle::Hidden => format!("[{}_HIDDEN]", kind),
        PlaceholderStyle::Removed => format!("[{}_REMOVED]", kind),
        PlaceholderStyle::Angle => format!("<{}>", kind),
    }
}

pub fn redact_note(input: &str, style: PlaceholderStyle) -> RedactionResult {
    let mut counts = RedactionCounts::default();
    let mut redacted = input.to_string();

    redacted = NAME_RE
        .replace_all(&redacted, |caps: &regex::Captures| {
            counts.names += 1;
            format!("{}: {}", &caps["label"], placeholder("NAME", style))
        })
        .into_owned();

    redacted = EMAIL_RE
        .replace_all(&redacted, |_: &regex::Captures| {
            counts.emails += 1;
            placeholder("EMAIL", style)
        })
        .into_owned();

    redacted = PHONE_RE
        .replace_all(&redacted, |_: &regex::Captures| {
            counts.phones += 1;
            placeholder("PHONE", style)
        })
        .into_owned();

    redacted = DOB_RE
        .replace_all(&redacted, |caps: &regex::Captures| {
            counts.dobs += 1;
            format!("{}: {}", &caps["label"], placeholder("DOB", style))
        })
        .into_owned();

    redacted = ID_LABEL_RE
        .replace_all(&redacted, |caps: &regex::Captures| {
            counts.ids += 1;
            format!("{}: {}", &caps["label"], placeholder("ID", style))
        })
        .into_owned();

    redacted = ADDRESS_LABEL_RE
        .replace_all(&redacted, |_: &regex::Captures| {
            counts.addresses += 1;
            format!("Address: {}", placeholder("ADDRESS", style))
        })
        .into_owned();

    redacted = ADDRESS_RE
        .replace_all(&redacted, |_: &regex::Captures| {
            counts.addresses += 1;
            placeholder("ADDRESS", style)
        })
        .into_owned();

    redacted = ID_GENERIC_RE
        .replace_all(&redacted, |_: &regex::Captures| {
            counts.ids += 1;
            placeholder("ID", style)
        })
        .into_owned();

    RedactionResult {
        redacted_text: redacted,
        counts,
        style,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_styles_render() {
        let kind = "EMAIL";
        assert_eq!(
            placeholder(kind, PlaceholderStyle::Protected),
            "[EMAIL_PROTECTED]"
        );
        assert_eq!(
            placeholder(kind, PlaceholderStyle::Masked),
            "[EMAIL_MASKED]"
        );
        assert_eq!(
            placeholder(kind, PlaceholderStyle::Hidden),
            "[EMAIL_HIDDEN]"
        );
        assert_eq!(
            placeholder(kind, PlaceholderStyle::Removed),
            "[EMAIL_REMOVED]"
        );
        assert_eq!(placeholder(kind, PlaceholderStyle::Angle), "<EMAIL>");
    }
}
