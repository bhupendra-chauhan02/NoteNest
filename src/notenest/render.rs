use super::types::{
    Clinician5Cs, ClinicianSoap, CoverageReport, NoteNestOutputs, PatientView, RedactionCounts,
    SummaryResult,
};

pub fn render_patient_view(view: &PatientView) -> String {
    let sections = [
        render_section("What you came in with", &what_you_came_in_with(view)),
        render_section(
            "What it could mean",
            std::slice::from_ref(&view.what_it_could_mean),
        ),
        render_section("What we found in your note", &what_we_found(view)),
        render_section("What to do next (checklist)", &view.next_steps),
        render_section("Questions to ask your clinician", &view.questions_to_ask),
        render_section("When to seek urgent care", &view.urgent_red_flags),
        render_section("Disclaimer", std::slice::from_ref(&view.disclaimer)),
    ];
    sections.join("\n\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClinicianMode {
    Soap,
    FiveCs,
    Both,
}

pub fn render_clinician_view(
    soap: &ClinicianSoap,
    five_cs: &Clinician5Cs,
    mode: ClinicianMode,
) -> String {
    match mode {
        ClinicianMode::Soap => render_soap(soap),
        ClinicianMode::FiveCs => render_five_cs(five_cs),
        ClinicianMode::Both => {
            let sections = [render_soap(soap), render_five_cs(five_cs)];
            sections.join("\n\n")
        }
    }
}

pub fn render_coverage(report: &CoverageReport) -> String {
    let protected_counts = format!(
        "names {}, phones {}, emails {}, dobs {}, ids {}, addresses {}",
        report.protected_counts.names,
        report.protected_counts.phones,
        report.protected_counts.emails,
        report.protected_counts.dobs,
        report.protected_counts.ids,
        report.protected_counts.addresses
    );

    format!(
        "Coverage summary\n- fields_found: {}\n- missing: {}\n- protected_counts: {}",
        report.fields_found,
        if report.fields_missing.is_empty() {
            "none".to_string()
        } else {
            report.fields_missing.join(", ")
        },
        protected_counts
    )
}

pub fn build_coverage_report(summary: &SummaryResult, counts: &RedactionCounts) -> CoverageReport {
    let fields = [
        ("chief_concern", &summary.chief_concern),
        ("duration", &summary.duration),
        ("symptoms", &summary.symptoms),
        ("pmh", &summary.pmh),
        ("meds", &summary.meds),
        ("allergies", &summary.allergies),
        ("vitals", &summary.vitals),
        ("tests", &summary.tests),
        ("assessment", &summary.assessment),
        ("plan", &summary.plan),
        ("context", &summary.context),
        ("concerns", &summary.concerns),
        ("coping", &summary.coping),
    ];

    let mut found = 0;
    let mut missing = Vec::new();
    for (label, items) in fields {
        if is_not_found(items) {
            missing.push(label.to_string());
        } else {
            found += 1;
        }
    }

    CoverageReport {
        fields_found: found,
        fields_missing: missing,
        protected_counts: counts.clone(),
    }
}

pub fn render_text_output(result: &NoteNestOutputs) -> String {
    render_text_output_with_mode(result, ClinicianMode::Both)
}

pub fn render_text_output_with_mode(result: &NoteNestOutputs, mode: ClinicianMode) -> String {
    let patient = render_patient_view(&result.patient_view);
    let clinician = render_clinician_view(&result.clinician_soap, &result.clinician_5cs, mode);
    let coverage = render_coverage(&result.coverage);
    format!(
        "Placeholder style: {}\n\nProtected note:\n{}\n\nPatient View:\n{}\n\nClinician View:\n{}\n\n{}",
        style_descriptor(result.placeholder_style.label()),
        result.protected_text,
        patient,
        clinician,
        coverage
    )
}

fn render_section(title: &str, items: &[String]) -> String {
    let lines: Vec<String> = items.iter().map(|item| format!("- {}", item)).collect();
    format!("{}\n{}", title, lines.join("\n"))
}

fn is_not_found(list: &[String]) -> bool {
    list.len() == 1 && list[0] == "Not found"
}

fn style_descriptor(label: &str) -> String {
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn render_soap(view: &ClinicianSoap) -> String {
    let sections = [
        render_section("SOAP - S", &view.s),
        render_section("SOAP - O", &view.o),
        render_section("SOAP - A", &view.a),
        render_section("SOAP - P", &view.p),
    ];
    sections.join("\n\n")
}

fn render_five_cs(view: &Clinician5Cs) -> String {
    let sections = [
        render_section(
            "5C's - Chief complaint",
            std::slice::from_ref(&view.chief_complaint),
        ),
        render_section("5C's - Course", &view.course),
        render_section("5C's - Context", &view.context),
        render_section("5C's - Concerns", &view.concerns),
        render_section("5C's - Coping", &view.coping),
    ];
    sections.join("\n\n")
}

fn what_you_came_in_with(view: &PatientView) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Main concern: {}", view.main_concern));
    if view.onset_duration != "Not found" {
        lines.push(format!("Duration: {}", view.onset_duration));
    }
    if !(view.triggers.is_empty() || (view.triggers.len() == 1 && view.triggers[0] == "Not found"))
    {
        lines.push(format!("Triggers: {}", view.triggers.join("; ")));
    }
    lines
}

fn what_we_found(view: &PatientView) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Symptoms: {}",
        view.what_we_found.symptoms.join("; ")
    ));
    lines.push(format!(
        "Negatives: {}",
        view.what_we_found.negatives.join("; ")
    ));
    lines.push(format!(
        "Medications: {}",
        view.what_we_found.medications.join("; ")
    ));
    lines.push(format!(
        "Allergies: {}",
        view.what_we_found.allergies.join("; ")
    ));
    lines.push(format!(
        "Tests/results: {}",
        view.what_we_found.tests_results.join("; ")
    ));
    lines.push(format!("Vitals: {}", view.what_we_found.vitals.join("; ")));
    lines
}
