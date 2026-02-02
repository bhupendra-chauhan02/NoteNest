use regex::Regex;

pub fn normalize_text(input: &str) -> String {
    let mut text = input.replace("\r\n", "\n");
    static MULTISPACE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"[\t ]{2,}").unwrap());
    static JUNK: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"(?i)template text|random junk|\.{5,}").unwrap());

    text = MULTISPACE.replace_all(&text, " ").to_string();
    text = JUNK.replace_all(&text, "").to_string();
    text
}
