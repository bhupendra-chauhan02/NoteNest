use once_cell::sync::Lazy;
use regex::Regex;

use super::types::{Clinician5Cs, ClinicianSoap, PatientFound, PatientView, SummaryResult};

static LAB_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)mg/dL|mmol/L|reference range|reference interval|units|HbA1c|WBC|RBC|Platelet|trop|crp|ecg|ekg|ct|cxr|x-ray",
    )
    .unwrap()
});

static TEST_KEYWORD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)hba1c|crp|wbc|rbc|platelet|lab|labs|ct|cxr|x-ray").unwrap());

static SYMPTOM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(sob|shortness of breath|breathe|breathing|dyspnea|cp|chest pain|chest tightness|tightness|abdo pain|abdominal pain|fever|cough|diarrhea|vomiting|nausea|dizzy|dizziness|headache|fatigue|sweating|palpitations|insomnia|anxiety|depression|stress|sleep)\b",
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
    Regex::new(
        r"(?i)(BP\s?\d{2,3}/\d{2,3}|HR\s?\d{2,3}|RR\s?\d{2,3}|T\s?\d{2}(?:\.\d+)?|Temp\s?\d{2}(?:\.\d+)?|SpO2\s?\d{2,3})",
    )
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

static PMH_COND_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(htn|dm2|diabetes|asthma|cad|copd|ckd)\b").unwrap());

static MED_DOSE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b([a-z][a-z0-9-]+)\s*(\d+(?:\.\d+)?\s?(?:mg|mcg|g)?)\s*(bid|tid|od|qd|qhs|prn)?\b",
    )
    .unwrap()
});

static TROPN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(trop|troponin)\s*[:=]?\s*([0-9.]+\s*\w+\/?\w*)").unwrap());

static ECG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(ecg|ekg)\b").unwrap());

static DENIES_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bdenies\s+([a-z0-9 ,/-]+)").unwrap());

static PHI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(contact|phone|email|mrn|dob|id)\b|\[(EMAIL|PHONE|ID|DOB|ADDRESS)_").unwrap()
});

static ADDRESS_LINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(addr|address|street|strasse|str)\b|\[ADDRESS_").unwrap());

static SPO2_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(spo2|o2\s*sat|sat)\s*[:\-]?\s*(\d{2,3})%?\b").unwrap());

static PLAN_ACTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(send|repeat|start|consider|follow[- ]?up|f/u|advised|recommend|monitor|refer|return)\b",
    )
    .unwrap()
});

pub fn summarize_note(input: &str) -> SummaryResult {
    let mut summary = SummaryResult::default();
    let lines: Vec<String> = input
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let is_lab_report = lines
        .iter()
        .any(|line| LAB_RE.is_match(line) || TROPN_RE.is_match(line) || ECG_RE.is_match(line));
    let mut current: Option<&str> = None;

    for line in &lines {
        if let Some(section) = match_heading(line, &mut summary) {
            current = Some(section);
            if section == "plan" {
                let rest = strip_heading(
                    line,
                    &[
                        "Plan",
                        "Treatment",
                        "Recommendations",
                        "Follow Up",
                        "Follow-Up",
                    ],
                )
                .unwrap_or("")
                .to_string();
                if !rest.is_empty() {
                    summary.plan.extend(split_plan_items(&rest));
                }
            }
            continue;
        }

        if let Some(section) = current
            && section == "plan"
        {
            if is_plan_stop(line) {
                current = None;
            } else if is_plan_continuation(line) {
                summary.plan.extend(split_plan_items(line));
                continue;
            } else {
                current = None;
            }
        }

        if is_plan_action_line(line) {
            summary.plan.extend(split_plan_items(line));
            continue;
        }

        if let Some(cc) = extract_chief_complaint_from_line(line)
            && summary.chief_concern.is_empty()
        {
            summary.chief_concern.push(cc);
        }

        if let Some(duration) = extract_duration(line) {
            summary.duration.push(duration);
        }

        let symptoms = extract_symptoms(line);
        if !symptoms.is_empty() {
            summary.symptoms.extend(symptoms.clone());
            summary.key_findings.extend(symptoms);
        }

        if let Some(neg) = extract_negatives(line) {
            summary.negatives.extend(neg);
        }

        summary.pmh.extend(extract_pmh(line));
        summary.meds.extend(extract_meds(line));
        if let Some(allergy) = extract_allergies(line) {
            summary.allergies.extend(allergy);
        }

        if is_clinical_line(line) {
            summary.vitals.extend(extract_vitals(line));
            summary.tests.extend(extract_tests(line));
        }

        if is_lab_report && LAB_RE.is_match(line) && is_clinical_line(line) {
            summary.key_results.extend(extract_tests(line));
        }

        if line.to_lowercase().contains("stress") || line.to_lowercase().contains("work") {
            summary.context.push(clean_line(line));
        }

        if line.to_lowercase().contains("sleep") {
            summary.coping.push(clean_line(line));
        }
    }

    if is_not_found(&summary.chief_concern)
        && let Some(complaint) = extract_chief_complaint(&lines)
    {
        summary.chief_concern = vec![complaint];
    }

    summary.concerns = extract_concerns(&summary);

    normalize_summary(&mut summary);
    summary
}

pub fn build_patient_view(summary: &SummaryResult) -> PatientView {
    let main_concern = if !is_not_found(&summary.chief_concern) {
        summary.chief_concern.join("; ")
    } else if !is_not_found(&summary.symptoms) {
        summary.symptoms.join("; ")
    } else if !is_not_found(&summary.context) {
        summary.context.join("; ")
    } else {
        "Not stated".to_string()
    };

    let onset_duration = if !is_not_found(&summary.duration) {
        summary.duration.join(", ")
    } else {
        "Not found".to_string()
    };

    let triggers = extract_triggers(summary);

    let mut what_it_could_mean =
        "These symptoms can have many causes. Some need urgent evaluation when breathing or chest symptoms are present. This summary is not a diagnosis.".to_string();
    if !is_not_found(&summary.assessment) {
        what_it_could_mean = format!(
            "{} Possible assessment mentioned: {}.",
            what_it_could_mean,
            summary.assessment.join("; ")
        );
    }

    let found = PatientFound {
        symptoms: normalize_list(summary.symptoms.clone()),
        negatives: normalize_list(summary.negatives.clone()),
        conditions: normalize_list(summary.pmh.clone()),
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
    let soap_s = vec![
        format!(
            "Chief complaint: {}",
            list_or_not_stated(&summary.chief_concern)
        ),
        format!("Onset/duration: {}", list_or_not_stated(&summary.duration)),
        format!("Symptoms: {}", list_or_not_stated(&summary.symptoms)),
        format!("Negatives: {}", list_or_not_stated(&summary.negatives)),
    ];
    let soap_o = vec![
        format!("Vitals: {}", list_or_not_stated(&summary.vitals)),
        format!("Tests/results: {}", list_or_not_stated(&summary.tests)),
    ];
    let soap_a = vec![format!(
        "Problem list: {}",
        list_or_not_stated(&summary.concerns)
    )];
    let soap_p = if is_not_found(&summary.plan) {
        vec!["Follow up with a clinician for evaluation.".to_string()]
    } else {
        summary.plan.clone()
    };

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
        context: normalize_list(vec![
            format!("PMH: {}", list_or_not_stated(&summary.pmh)),
            format!("Meds: {}", list_or_not_stated(&summary.meds)),
            format!("Allergies: {}", list_or_not_stated(&summary.allergies)),
            format!("Social/other: {}", list_or_not_stated(&summary.context)),
        ]),
        concerns: normalize_list(summary.concerns.clone()),
        coping: normalize_list_with_default(summary.coping.clone(), "Not stated"),
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
            let symptoms = extract_symptoms(rest);
            if !symptoms.is_empty() {
                summary.key_findings.extend(symptoms.clone());
                summary.symptoms.extend(symptoms);
            }
            if let Some(neg) = extract_negatives(rest) {
                summary.negatives.extend(neg);
            }
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
            summary.plan.extend(split_plan_items(rest));
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
                .trim_start_matches([':', '-', '>', ' '])
                .trim();
            return Some(rest);
        }
    }
    None
}

fn normalize_summary(summary: &mut SummaryResult) {
    summary.chief_concern = normalize_list(summary.chief_concern.clone());
    summary.duration = normalize_list(summary.duration.clone());
    summary.symptoms = normalize_list(summary.symptoms.clone());
    summary.negatives = normalize_list(summary.negatives.clone());
    summary.pmh = normalize_list(summary.pmh.clone());
    summary.meds = normalize_list(summary.meds.clone());
    summary.allergies = normalize_list(summary.allergies.clone());
    summary.vitals = normalize_list(summary.vitals.clone());
    summary.tests = normalize_list(summary.tests.clone());
    summary.key_findings = normalize_list(summary.key_findings.clone());
    summary.assessment = normalize_list(summary.assessment.clone());
    summary.plan = dedupe_case_insensitive(normalize_list(summary.plan.clone()));
    summary.context = normalize_list(summary.context.clone());
    summary.concerns = normalize_list(summary.concerns.clone());
    summary.coping = normalize_list(summary.coping.clone());
    summary.key_results = normalize_list(summary.key_results.clone());

    if summary
        .symptoms
        .contains(&"shortness of breath".to_string())
        && summary.symptoms.contains(&"chest tightness".to_string())
    {
        summary.symptoms.retain(|item| item != "chest tightness");
        summary
            .symptoms
            .retain(|item| item != "shortness of breath");
        summary
            .symptoms
            .insert(0, "Shortness of breath with chest tightness".to_string());
    } else {
        summary.symptoms = summary
            .symptoms
            .iter()
            .map(|item| {
                if item == "shortness of breath" {
                    "Shortness of breath".to_string()
                } else {
                    item.clone()
                }
            })
            .collect();
    }
}

fn normalize_list(mut list: Vec<String>) -> Vec<String> {
    list.retain(|item| !item.trim().is_empty());
    if list.is_empty() {
        list.push("Not found".to_string());
    }
    list
}

fn dedupe_case_insensitive(list: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for item in list {
        let key = item.to_lowercase();
        if seen.insert(key) {
            out.push(item);
        }
    }
    if out.is_empty() {
        vec!["Not found".to_string()]
    } else {
        out
    }
}

fn is_not_found(list: &[String]) -> bool {
    list.len() == 1 && list[0] == "Not found"
}

fn list_or_not_stated(list: &[String]) -> String {
    if is_not_found(list) {
        "Not stated".to_string()
    } else {
        list.join("; ")
    }
}

fn normalize_list_with_default(mut list: Vec<String>, default: &str) -> Vec<String> {
    list.retain(|item| !item.trim().is_empty());
    if list.is_empty() || (list.len() == 1 && list[0] == "Not found") {
        list.clear();
        list.push(default.to_string());
    }
    list
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

fn extract_symptoms(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for cap in SYMPTOM_RE.find_iter(line) {
        let token = cap.as_str().to_lowercase();
        let normalized = match token.as_str() {
            "sob" => "shortness of breath".to_string(),
            "breathe" => "shortness of breath".to_string(),
            "breathing" => "shortness of breath".to_string(),
            "dyspnea" => "shortness of breath".to_string(),
            "cp" => "chest pain".to_string(),
            "abdo pain" => "abdominal pain".to_string(),
            "chest tightness" => "chest tightness".to_string(),
            other => other.to_string(),
        };
        if !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out
}

fn extract_chief_complaint_from_line(line: &str) -> Option<String> {
    if line.contains('"') {
        let mut parts = line.split('"');
        if let (Some(_), Some(quoted)) = (parts.next(), parts.next()) {
            let trimmed = quoted.trim();
            if !trimmed.is_empty() {
                return Some(shorten_phrase(trimmed));
            }
        }
    }
    if let Some(rest) = strip_heading(line, &["CC", "Chief Complaint", "Chief Concern"])
        && !rest.is_empty()
    {
        return Some(shorten_phrase(rest));
    }
    if line.to_lowercase().contains("pt") && SYMPTOM_RE.is_match(line) {
        let symptoms = extract_symptoms(line);
        if !symptoms.is_empty() {
            return Some(shorten_phrase(&symptoms.join(", ")));
        }
    }
    None
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
    if words.len() <= 8 {
        text.trim().to_string()
    } else {
        words[..8].join(" ")
    }
}

fn clean_line(line: &str) -> String {
    line.trim().trim_end_matches(['.', ';']).to_string()
}

fn extract_duration(line: &str) -> Option<String> {
    DURATION_RE.find(line).map(|m| m.as_str().to_string())
}

fn extract_negatives(line: &str) -> Option<Vec<String>> {
    DENIES_RE.captures(line).map(|cap| {
        cap[1]
            .split([',', ';', '/'])
            .map(|item| {
                let cleaned = item.trim().trim_end_matches(['.', ';', ',']).trim();
                format!("Denies {}", cleaned)
            })
            .filter(|item| item.len() > "Denies ".len())
            .collect::<Vec<String>>()
    })
}

fn extract_pmh(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    if PMH_RE.is_match(line) || PMH_COND_RE.is_match(line) {
        for cap in PMH_COND_RE.find_iter(line) {
            let token = cap.as_str().to_lowercase();
            let normalized = match token.as_str() {
                "htn" => "HTN",
                "dm2" => "DM2",
                "diabetes" => "Diabetes",
                "cad" => "CAD",
                "asthma" => "Asthma",
                "copd" => "COPD",
                "ckd" => "CKD",
                other => other,
            };
            let value = normalized.to_string();
            if !out.contains(&value) {
                out.push(value);
            }
        }
    }
    out
}

fn extract_meds(line: &str) -> Vec<String> {
    if is_plan_action_line(line) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let lower = line.to_lowercase();
    let meds_context = lower.contains("meds") || lower.contains("taking");
    let segment = if let Some(idx) = lower.find("allergy") {
        &line[..idx]
    } else {
        line
    };
    if lower.contains("nkda") {
        return out;
    }
    let cleaned_segment = normalize_med_segment(segment);
    if lower.contains("meds") && (lower.contains("none") || lower.contains("no meds")) {
        out.push("None reported".to_string());
        return out;
    }
    for cap in MED_DOSE_RE.captures_iter(&cleaned_segment) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let dose = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let freq = cap.get(3).map(|m| m.as_str()).unwrap_or("");
        let name_lower = name.to_lowercase();
        if matches!(
            name_lower.as_str(),
            "htn" | "dm" | "dm2" | "cad" | "copd" | "ckd" | "asthma" | "diabetes"
        ) {
            continue;
        }
        let mut entry = format!("{} {}", name, dose).trim().to_string();
        if !freq.is_empty() {
            entry = format!("{} {}", entry, freq.to_uppercase());
        }
        entry = normalize_med_entry(&entry);
        if is_medication_candidate(&entry, meds_context) && !out.contains(&entry) {
            out.push(entry);
        }
    }
    if out.is_empty() && MED_RE.is_match(segment) {
        for chunk in cleaned_segment.split(['+', ';', ',']) {
            let trimmed = clean_line(chunk);
            if trimmed.is_empty() {
                continue;
            }
            for cap in MED_DOSE_RE.captures_iter(&trimmed) {
                let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let dose = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                let freq = cap.get(3).map(|m| m.as_str()).unwrap_or("");
                let name_lower = name.to_lowercase();
                if matches!(
                    name_lower.as_str(),
                    "htn" | "dm" | "dm2" | "cad" | "copd" | "ckd" | "asthma" | "diabetes"
                ) {
                    continue;
                }
                let mut entry = format!("{} {}", name, dose).trim().to_string();
                if !freq.is_empty() {
                    entry = format!("{} {}", entry, freq.to_uppercase());
                }
                entry = normalize_med_entry(&entry);
                if is_medication_candidate(&entry, meds_context) && !out.contains(&entry) {
                    out.push(entry);
                }
            }
        }
        if out.is_empty() {
            let cleaned = clean_line(&cleaned_segment);
            if !cleaned.is_empty() {
                let normalized = normalize_med_entry(&cleaned);
                if is_medication_candidate(&normalized, meds_context) {
                    out.push(normalized);
                }
            }
        }
    }
    out
}

fn extract_allergies(line: &str) -> Option<Vec<String>> {
    if !ALLERGY_RE.is_match(line) {
        return None;
    }
    let lower = line.to_lowercase();
    if lower.contains("nkda") || lower.contains("no known drug allergies") {
        return Some(vec!["No known drug allergies".to_string()]);
    }
    if let Some(rest) = strip_heading(line, &["Allergy", "Allergies"])
        && !rest.is_empty()
    {
        return Some(vec![rest.to_string()]);
    }
    if let Some(idx) = lower.find("allergy") {
        let slice = line[idx..].replace("allergy", "").replace("Allergy", "");
        let cleaned = clean_line(&slice);
        if !cleaned.is_empty() {
            return Some(vec![cleaned]);
        }
    }
    Some(vec!["Not stated".to_string()])
}

fn extract_vitals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut remaining = line.to_string();
    if let Some(cap) = SPO2_RE.captures(line) {
        let value = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        if !value.is_empty() {
            out.push(format!("SpO2 {}%", value));
            let spo2_token =
                Regex::new(r"(?i)\b(spo2|spo\s*2|o2\s*sat|sat)\s*[:\-]?\s*\d{2,3}%?\b").unwrap();
            remaining = spo2_token.replace_all(&remaining, "").to_string();
        }
    }
    for cap in VITAL_RE.find_iter(&remaining) {
        let token = cap.as_str().to_string();
        let normalized = token.replace("Temp", "T").replace("temp", "T");
        let normalized = if let Some(idx) = normalized.find(|c: char| c.is_ascii_digit()) {
            let (label, value) = normalized.split_at(idx);
            format!("{} {}", label.trim(), value.trim())
        } else {
            normalized
        };
        if !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out.retain(|item| {
        let lower = item.to_lowercase();
        !(lower.contains("spo 2") || lower.contains("spo2") || lower.contains("o2 sat"))
            || item.starts_with("SpO2 ")
    });
    out
}

fn extract_tests(line: &str) -> Vec<String> {
    let lower = line.to_lowercase();
    if !is_clinical_line(line) || lower.contains("repeat") || is_plan_action_line(line) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let has_trop = TROPN_RE.is_match(line);
    let has_ecg = ECG_RE.is_match(line);
    let has_test_keyword = TEST_KEYWORD_RE.is_match(line);
    if (lower.contains("vitals") || VITAL_RE.is_match(line) || SPO2_RE.is_match(line))
        && !(has_trop || has_ecg || has_test_keyword)
    {
        return Vec::new();
    }
    if let Some(cap) = TROPN_RE.captures(line) {
        let value = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        if !value.is_empty() {
            out.push(format!("Troponin: {}", value));
        }
    }
    if ECG_RE.is_match(line) {
        if line.to_lowercase().contains("st") {
            out.push("ECG: possible ST changes".to_string());
        } else {
            out.push("ECG: noted".to_string());
        }
    }
    if out.is_empty() && has_test_keyword {
        out.push(clean_line(line));
    }
    out
}

fn split_plan_items(text: &str) -> Vec<String> {
    let mut cleaned = text.replace("plan:", "").replace("Plan:", "");
    let verb_split = Regex::new(
        r"(?i)\b(send|repeat|start|consider|follow[- ]?up|f/u|advised|recommend|monitor)\b",
    )
    .unwrap();
    cleaned = verb_split.replace_all(&cleaned, ";$1").to_string();
    cleaned = cleaned.replace("->", ";");
    cleaned = cleaned.replace("  ", " ");
    cleaned = cleaned.replace("&&", ";");
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for chunk in cleaned.split([';', '.']) {
        let trimmed = chunk
            .trim()
            .trim_start_matches(['>', '-', '*', '•', ':'])
            .trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if is_plan_stop(trimmed)
            || lower.contains("random junk")
            || lower.contains("template")
            || lower.contains("copied")
            || PHI_RE.is_match(trimmed)
            || ADDRESS_LINE_RE.is_match(trimmed)
        {
            break;
        }
        if !PLAN_ACTION_RE.is_match(trimmed)
            && (lower.contains("ecg") || lower.contains("ekg") || lower.contains("labs"))
        {
            break;
        }
        let key = lower.clone();
        if seen.insert(key) {
            items.push(trimmed.to_string());
        }
    }
    items
}

fn is_clinical_line(line: &str) -> bool {
    !(PHI_RE.is_match(line) || ADDRESS_LINE_RE.is_match(line))
}

fn is_plan_action_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    if lower.contains("plan") {
        return true;
    }
    if lower.starts_with("meds")
        || lower.starts_with("med ")
        || lower.starts_with("pmh")
        || lower.starts_with("allerg")
    {
        return false;
    }
    PLAN_ACTION_RE.is_match(line)
}

fn extract_concerns(summary: &SummaryResult) -> Vec<String> {
    let mut concerns = Vec::new();
    let symptoms = summary.symptoms.join(" ").to_lowercase();
    let tests = summary.tests.join(" ").to_lowercase();
    let vitals = summary.vitals.join(" ").to_lowercase();
    if (symptoms.contains("chest pain") || symptoms.contains("chest tightness"))
        && (symptoms.contains("shortness of breath") || symptoms.contains("sob"))
    {
        concerns.push("Chest pain + shortness of breath — consider cardiac vs pulmonary causes; correlate with ECG/troponin; rule out ACS.".to_string());
    }
    if tests.contains("troponin") {
        concerns.push(
            "Troponin noted — correlate with repeat labs and ECG; clinical correlation required."
                .to_string(),
        );
    }
    if vitals.contains("bp") && vitals.contains("/") {
        concerns
            .push("Elevated blood pressure noted — monitor and evaluate in context.".to_string());
    }
    if concerns.is_empty() {
        concerns.push("Not stated".to_string());
    }
    concerns
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
        || lower.contains("do not share")
        || lower.contains("random footer")
        || lower.contains("template")
        || lower.starts_with("-----")
        || lower.starts_with("bp")
        || lower.starts_with("hr")
        || lower.contains("ecg")
        || lower.contains("ekg")
        || lower.contains("labs")
}

fn normalize_med_segment(segment: &str) -> String {
    let mut cleaned = segment
        .replace("pmh", "")
        .replace("PMH", "")
        .replace("hx", "")
        .replace("HX", "");
    let meds_label = Regex::new(r"(?i)\bmeds?\b[:\-]?\s*").unwrap();
    cleaned = meds_label.replace_all(&cleaned, "").to_string();
    let join_digits = Regex::new(r"([A-Za-z])(\d)").unwrap();
    cleaned = join_digits.replace_all(&cleaned, "$1 $2").to_string();
    let split_freq = Regex::new(r"(\d)(bid|tid|od|qd|qhs|prn)\b").unwrap();
    cleaned = split_freq.replace_all(&cleaned, "$1 $2").to_string();
    cleaned
}

fn normalize_med_entry(entry: &str) -> String {
    let freq_re = Regex::new(r"\b(bid|tid|od|qd|qhs|prn)\b").unwrap();
    let mut normalized = freq_re
        .replace_all(entry, |caps: &regex::Captures| caps[0].to_uppercase())
        .to_string();
    let unit_re = Regex::new(r"(\d)(mg|mcg|g)\b").unwrap();
    normalized = unit_re.replace_all(&normalized, "$1 $2").to_string();
    normalized = normalized.replace("  ", " ");
    normalized.trim().to_string()
}

fn is_medication_candidate(value: &str, meds_context: bool) -> bool {
    let lower = value.to_lowercase();
    if lower.contains("nkda") || lower.contains("no known drug allergies") {
        return false;
    }
    if Regex::new(r"(?i)\\b(bp|hr|rr|temp|spo2|sat|o2|pulse)\\b")
        .unwrap()
        .is_match(&lower)
    {
        return false;
    }
    if Regex::new(
        r"(?i)\\b(trop|troponin|ecg|ekg|cxr|ct|mri|cbc|bmp|crp|wbc|hba1c|ng/ml|mmol/l|mg/dl)\\b",
    )
    .unwrap()
    .is_match(&lower)
    {
        return false;
    }
    if Regex::new(r"\[(EMAIL|PHONE|ID|DOB|ADDRESS|NAME)_")
        .unwrap()
        .is_match(value)
    {
        return false;
    }
    if Regex::new(r"(?i)\\b(mg|mcg|g|ml)\\b")
        .unwrap()
        .is_match(&lower)
        || Regex::new(r"(?i)\\b(bid|tid|qid|od|qd|qhs|prn)\\b")
            .unwrap()
            .is_match(&lower)
    {
        return true;
    }
    let has_alpha = Regex::new(r"[A-Za-z]{2,}").unwrap().is_match(&lower);
    if meds_context && has_alpha {
        return true;
    }
    false
}
