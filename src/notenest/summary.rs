use once_cell::sync::Lazy;
use regex::Regex;

use super::types::{Clinician5Cs, ClinicianSoap, PatientFound, PatientView, SummaryResult};

static LAB_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)mg/dL|mmol/L|reference range|reference interval|units|HbA1c|WBC|RBC|Platelet|trop|crp|ecg|ekg",
    )
    .unwrap()
});

static SYMPTOM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(sob|shortness of breath|breathe|breathing|dyspnea|cp|chest pain|tightness|abdo pain|abdominal pain|fatigue|nausea|vomiting|diarrhea|cough|fever|dizzy|dizziness|headache)\b",
    )
    .unwrap()
});

static DURATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(x\s?\d+\s?(d|day|days|w|wk|week|weeks|mo|month|months)|for\s+\d+\s?(d|day|days|w|wk|week|weeks|mo|month|months)|since\s+\w+|started\s+last\s+\w+)",
    )
    .unwrap()
});

static VITAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(BP\s?\d{2,3}/\d{2,3}|HR\s?\d{2,3}|RR\s?\d{2,3}|T\s?\d{2,2}\.\d|Temp\s?\d{2,2}\.\d|SpO2\s?\d{2,3})")
        .unwrap()
});

static MED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(meds?|taking|metformin|ramipril|lisinopril|amlodipine|ibuprofen|ibu|asa|statin|prazosin)\b",
    )
    .unwrap()
});

static ALLERGY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(nkda|allergy|allergies|penicillin)\b").unwrap());

static PMH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(pmh|hx)\b").unwrap());

pub fn summarize_note(input: &str) -> SummaryResult {
    let mut summary = SummaryResult::default();
    let lines: Vec<String> = input
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let is_lab_report = lines.iter().any(|line| LAB_RE.is_match(line));
    let mut current: Option<&str> = None;

    for line in &lines {
        if let Some(section) = match_heading(line, &mut summary) {
            current = Some(section);
            continue;
        }

        if let Some(section) = current {
            if section == "plan" {
                if is_plan_stop(line) {
                    current = None;
                } else if is_plan_continuation(line) {
                    push_to_section(&mut summary, section, clean_line(line));
                    continue;
                } else {
                    current = None;
                }
            } else {
                let cleaned = line
                    .trim_start_matches(|c: char| c == '-' || c == '*' || c.is_numeric())
                    .trim()
                    .to_string();
                if !cleaned.is_empty() {
                    push_to_section(&mut summary, section, cleaned);
                }
                continue;
            }
        }

        if is_lab_report && LAB_RE.is_match(line) {
            summary.key_results.push(line.clone());
            summary.tests.push(line.clone());
        }

        if !line.to_lowercase().contains("denies") {
            let symptoms = extract_symptoms(line);
            if !symptoms.is_empty() {
                summary.symptoms.extend(symptoms.clone());
                summary.key_findings.extend(symptoms);
            }
        }

        if let Some(m) = DURATION_RE.find(line) {
            summary.duration.push(m.as_str().to_string());
        }

        for cap in VITAL_RE.find_iter(line) {
            summary.vitals.push(cap.as_str().to_string());
        }

        if LAB_RE.is_match(line) && !summary.tests.contains(line) {
            summary.tests.push(line.clone());
        }

        if MED_RE.is_match(line) {
            summary.meds.push(clean_line(line));
        }

        if ALLERGY_RE.is_match(line) {
            if line.to_lowercase().contains("nkda")
                || line.to_lowercase().contains("no known drug allergies")
            {
                summary.allergies.clear();
                summary.allergies.push("NKDA".to_string());
            } else {
                summary.allergies.push(clean_line(line));
            }
        }

        if PMH_RE.is_match(line) {
            summary.pmh.push(clean_line(line));
        }

        if line.to_lowercase().contains("stress") {
            summary.context.push(line.clone());
        }

        if line.to_lowercase().contains("denies") {
            summary.concerns.push(line.clone());
        }

        if line.to_lowercase().contains("sleep") {
            summary.coping.push(line.clone());
        }
    }

    if is_not_found(&summary.chief_concern)
        && let Some(complaint) = extract_chief_complaint(&lines)
    {
        summary.chief_concern = vec![complaint];
    }

    normalize_summary(&mut summary);
    summary
}

pub fn build_patient_view(summary: &SummaryResult) -> PatientView {
    let main_concern = if !is_not_found(&summary.chief_concern) {
        summary.chief_concern.join("; ")
    } else if !is_not_found(&summary.symptoms) {
        summary.symptoms.join("; ")
    } else {
        "Not found".to_string()
    };

    let onset_duration = if !is_not_found(&summary.duration) {
        summary.duration.join(", ")
    } else {
        "Not found".to_string()
    };

    let triggers = extract_triggers(summary);

    let mut what_it_could_mean =
        "These symptoms can have many causes. This summary is not a diagnosis.".to_string();
    if !is_not_found(&summary.assessment) {
        what_it_could_mean = format!(
            "{} Possible assessment mentioned: {}.",
            what_it_could_mean,
            summary.assessment.join("; ")
        );
    }

    let found = PatientFound {
        symptoms: normalize_list(summary.symptoms.clone()),
        negatives: normalize_list(collect_negatives(summary)),
        medications: normalize_list(summary.meds.clone()),
        allergies: normalize_list(summary.allergies.clone()),
        tests_results: normalize_list(summary.tests.clone()),
        vitals: normalize_list(summary.vitals.clone()),
    };

    let mut next_steps = Vec::new();
    if !is_not_found(&summary.plan) {
        next_steps.extend(summary.plan.clone());
    }
    next_steps.push("Confirm timing, doses, and follow-up details with your clinician.".into());

    let questions = vec![
        "What is the most likely cause of my symptoms?".into(),
        "What warning signs should make me seek help immediately?".into(),
        "What tests are still pending, and what do they mean?".into(),
        "What is my follow-up plan and timeline?".into(),
    ];

    let urgent_red_flags = vec![
        "Worsening chest pain or pressure".into(),
        "Severe difficulty breathing".into(),
        "Fainting or confusion".into(),
        "Blue lips/face, or new weakness on one side".into(),
    ];

    PatientView {
        main_concern,
        onset_duration,
        triggers,
        what_it_could_mean,
        what_we_found: found,
        next_steps: normalize_list(next_steps),
        questions_to_ask: questions,
        urgent_red_flags,
        disclaimer: "This summary is for informational use and does not replace medical advice."
            .to_string(),
    }
}

pub fn build_clinician_soap(summary: &SummaryResult) -> ClinicianSoap {
    let soap_s = merge_sections(&[
        ("CC", &summary.chief_concern),
        ("Symptoms", &summary.symptoms),
        ("Duration", &summary.duration),
        ("Context", &summary.context),
        ("Concerns", &summary.concerns),
    ]);
    let soap_o = merge_sections(&[
        ("Vitals", &summary.vitals),
        ("Tests", &summary.tests),
        ("PMH", &summary.pmh),
        ("Meds", &summary.meds),
        ("Allergies", &summary.allergies),
    ]);
    let soap_a = if is_not_found(&summary.assessment) {
        vec!["Assessment not explicitly stated.".to_string()]
    } else {
        summary.assessment.clone()
    };
    let soap_p = normalize_list(summary.plan.clone());

    ClinicianSoap {
        s: soap_s,
        o: soap_o,
        a: soap_a,
        p: soap_p,
    }
}

pub fn build_clinician_5cs(summary: &SummaryResult) -> Clinician5Cs {
    Clinician5Cs {
        chief_complaint: if !is_not_found(&summary.chief_concern) {
            summary.chief_concern.join("; ")
        } else {
            "Not found".to_string()
        },
        course: normalize_list(summary.duration.clone()),
        context: normalize_list(summary.context.clone()),
        concerns: normalize_list(summary.concerns.clone()),
        coping: normalize_list(summary.coping.clone()),
    }
}

fn match_heading(line: &str, summary: &mut SummaryResult) -> Option<&'static str> {
    if let Some(rest) = strip_heading(
        line,
        &["Chief Complaint", "Chief Concern", "CC", "Reason for Visit"],
    ) {
        if !rest.is_empty() {
            summary.chief_concern.push(rest.to_string());
        }
        return Some("chief_concern");
    }

    if let Some(rest) = strip_heading(line, &["HPI", "Symptoms", "Findings"]) {
        if !rest.is_empty() {
            summary.key_findings.push(rest.to_string());
            summary.symptoms.push(rest.to_string());
        }
        return Some("symptoms");
    }

    if let Some(rest) = strip_heading(line, &["Assessment", "Impression", "Diagnosis"]) {
        if !rest.is_empty() {
            summary.assessment.push(rest.to_string());
        }
        return Some("assessment");
    }

    if let Some(rest) = strip_heading(
        line,
        &[
            "Plan",
            "Treatment",
            "Recommendations",
            "Follow Up",
            "Follow-Up",
        ],
    ) {
        if !rest.is_empty() {
            summary.plan.push(rest.to_string());
        }
        return Some("plan");
    }

    None
}

fn strip_heading<'a>(line: &'a str, headings: &[&str]) -> Option<&'a str> {
    let lower = line.to_lowercase();
    for heading in headings {
        let heading_lower = heading.to_lowercase();
        if lower.starts_with(&heading_lower) {
            let rest = line[heading.len()..]
                .trim_start_matches([':', '-', ' '])
                .trim();
            return Some(rest);
        }
    }
    None
}

fn push_to_section(summary: &mut SummaryResult, section: &str, value: String) {
    match section {
        "chief_concern" => summary.chief_concern.push(value),
        "symptoms" => summary.symptoms.push(value),
        "assessment" => summary.assessment.push(value),
        "plan" => summary.plan.push(value),
        _ => {}
    }
}

fn normalize_summary(summary: &mut SummaryResult) {
    summary.chief_concern = normalize_list(summary.chief_concern.clone());
    summary.duration = normalize_list(summary.duration.clone());
    summary.symptoms = normalize_list(summary.symptoms.clone());
    summary.pmh = normalize_list(summary.pmh.clone());
    summary.meds = normalize_list(summary.meds.clone());
    summary.allergies = normalize_list(summary.allergies.clone());
    summary.vitals = normalize_list(summary.vitals.clone());
    summary.tests = normalize_list(summary.tests.clone());
    summary.key_findings = normalize_list(summary.key_findings.clone());
    summary.assessment = normalize_list(summary.assessment.clone());
    summary.plan = normalize_list(summary.plan.clone());
    summary.context = normalize_list(summary.context.clone());
    summary.concerns = normalize_list(summary.concerns.clone());
    summary.coping = normalize_list(summary.coping.clone());
    summary.key_results = normalize_list(summary.key_results.clone());
}

fn normalize_list(mut list: Vec<String>) -> Vec<String> {
    list.retain(|item| !item.trim().is_empty());
    if list.is_empty() {
        list.push("Not found".to_string());
    }
    list
}

fn is_not_found(list: &[String]) -> bool {
    list.len() == 1 && list[0] == "Not found"
}

fn collect_negatives(summary: &SummaryResult) -> Vec<String> {
    let mut negatives = Vec::new();
    for concern in &summary.concerns {
        if concern.to_lowercase().contains("denies") {
            negatives.push(concern.clone());
        }
    }
    negatives
}

fn extract_triggers(summary: &SummaryResult) -> Vec<String> {
    let mut triggers = Vec::new();
    for line in summary.symptoms.iter().chain(summary.context.iter()) {
        let lower = line.to_lowercase();
        if (lower.contains("worse") || lower.contains("exertion") || lower.contains("stairs"))
            && !triggers.contains(line)
        {
            triggers.push(line.clone());
        }
    }
    if triggers.is_empty() {
        triggers.push("Not found".to_string());
    }
    triggers
}

fn merge_sections(sections: &[(&str, &Vec<String>)]) -> Vec<String> {
    let mut merged = Vec::new();
    for (label, items) in sections {
        if !is_not_found(items) {
            merged.push(format!("{}: {}", label, items.join("; ")));
        }
    }
    if merged.is_empty() {
        merged.push("Not found".to_string());
    }
    merged
}

fn extract_symptoms(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for cap in SYMPTOM_RE.find_iter(line) {
        let token = cap.as_str().to_lowercase();
        let normalized = match token.as_str() {
            "sob" => "shortness of breath".to_string(),
            "cp" => "chest pain".to_string(),
            "abdo pain" => "abdominal pain".to_string(),
            other => other.to_string(),
        };
        if !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out
}

fn extract_chief_complaint(lines: &[String]) -> Option<String> {
    for line in lines.iter().take(2) {
        if let Some(rest) = strip_heading(line, &["CC", "Chief Complaint", "Chief Concern"]) {
            return Some(shorten_phrase(rest));
        }
        let symptoms = extract_symptoms(line);
        if !symptoms.is_empty() {
            return Some(shorten_phrase(&symptoms.join(", ")));
        }
    }
    for line in lines {
        if line.contains('"') {
            let mut parts = line.split('"');
            if let (Some(_), Some(quoted)) = (parts.next(), parts.next())
                && !quoted.trim().is_empty()
            {
                return Some(shorten_phrase(quoted.trim()));
            }
        }
        if line.to_lowercase().contains("can't breathe") {
            return Some("shortness of breath".to_string());
        }
        let symptoms = extract_symptoms(line);
        if !symptoms.is_empty() {
            return Some(shorten_phrase(&symptoms.join(", ")));
        }
    }
    None
}

fn shorten_phrase(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= 12 {
        text.trim().to_string()
    } else {
        words[..12].join(" ")
    }
}

fn clean_line(line: &str) -> String {
    line.trim().trim_end_matches(['.', ';']).to_string()
}

fn is_plan_continuation(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('-')
        || trimmed.starts_with('*')
        || trimmed.starts_with("->")
        || trimmed.len() < line.len()
}

fn is_plan_stop(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.starts_with("addr")
        || lower.starts_with("address")
        || lower.contains("[address_protected]")
        || lower.starts_with("dob")
        || lower.starts_with("mrn")
        || lower.starts_with("pmh")
        || lower.starts_with("meds")
        || lower.starts_with("allerg")
        || lower.starts_with("bp")
        || lower.starts_with("hr")
        || lower.contains("ecg")
        || lower.contains("ekg")
        || lower.contains("labs")
        || lower.contains("trop")
}
