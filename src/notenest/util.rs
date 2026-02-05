use regex::Regex;

pub fn normalize_text(input: &str) -> String {
    let mut text = input.replace("\r\n", "\n");
    static MULTISPACE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"[\t ]{2,}").unwrap());
    static JUNK: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(
            r"(?i)template text|random junk|random footer|copied template|lorem|do not share|outside hospital|footer|\.{5,}",
        )
        .unwrap()
    });
    static REPEAT_PUNCT: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"[?!.,]{2,}").unwrap());
    static TIMESTAMP: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"^\d{1,2}:\d{2}").unwrap());
    static TRIAGE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"(?i)^triage note\s*[-—]*\s*").unwrap());
    static SECTION_SPLIT: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?i)\b(pmh|hx|meds?|vitals?|plan|allerg(?:y|ies)?|nkda|addr|address)\b")
            .unwrap()
    });
    static NON_LETTER: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"[^A-Za-z]").unwrap());

    text = text.replace(';', "\n");
    text = SECTION_SPLIT.replace_all(&text, "\n$1").to_string();
    text = REPEAT_PUNCT
        .replace_all(&text, |caps: &regex::Captures| {
            caps.get(0)
                .and_then(|m| m.as_str().chars().next())
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .to_string();
    text = MULTISPACE.replace_all(&text, " ").to_string();
    text = JUNK.replace_all(&text, "").to_string();

    let mut cleaned_lines = Vec::new();
    for raw_line in text.lines() {
        let mut line = raw_line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if TIMESTAMP.is_match(&line) {
            line = TIMESTAMP.replace(&line, "").trim().to_string();
        }
        line = TRIAGE.replace(&line, "").trim().to_string();
        let lower = line.to_lowercase();
        if lower.contains("do not share") || lower.contains("outside hospital") {
            continue;
        }
        let letters_only = NON_LETTER.replace_all(&line, "");
        if letters_only.is_empty() && (line.contains('-') || line.contains('.')) {
            continue;
        }
        let has_clinical_signal = lower.contains("bp")
            || lower.contains("hr")
            || lower.contains("rr")
            || lower.contains("spo2")
            || lower.contains("trop")
            || lower.contains("ecg")
            || lower.contains("med")
            || lower.contains("pmh")
            || lower.contains("plan")
            || lower.contains("sob")
            || lower.contains("mrn")
            || lower.contains("dob")
            || lower.contains("addr")
            || lower.contains("id")
            || lower.contains("@");
        let letters = NON_LETTER.replace_all(&line, "").len();
        if !has_clinical_signal && (letters == 0 || letters * 2 < line.len()) {
            continue;
        }
        cleaned_lines.push(line);
    }

    cleaned_lines.join("\n")
}
