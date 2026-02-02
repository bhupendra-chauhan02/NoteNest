use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use clap::Parser;

use notenest::cli::{
    BatchArgs, CloakCommand, CloakConfigCommand, CloakOutputFormat, Command, CommandArgs,
    ConvertArgs, LegacyArgs, OutputFormat, ReportFormat,
};
use notenest::cloak::CloakEngine;
use notenest::notenest::{ClinicianMode, render_text_output_with_mode};
use notenest::util::fs::{list_files, read_to_string, write_string};
use notenest::{PlaceholderStyle, process_note};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1
        && matches!(
            args[1].as_str(),
            "convert" | "batch" | "cloak" | "self-check"
        )
    {
        let parsed = CommandArgs::parse();
        if let Err(err) = run_command(parsed.command) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    } else {
        let legacy = LegacyArgs::parse_from(args);
        if let Err(err) = run_legacy(legacy) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn run_legacy(args: LegacyArgs) -> Result<(), String> {
    let style = PlaceholderStyle::from_str(&args.placeholder_style)?;
    let input = read_input(&args.input)?;
    if input.trim().is_empty() {
        return Err("input is empty".to_string());
    }
    let result = process_note(&input, style);
    match args.format {
        OutputFormat::Json => {
            let payload = serde_json::to_string_pretty(&result)
                .map_err(|err| format!("json serialization failed: {}", err))?;
            println!("{}", payload);
        }
        OutputFormat::Text => {
            println!(
                "{}",
                render_text_output_with_mode(&result, ClinicianMode::Both)
            );
        }
        OutputFormat::Both => {
            println!(
                "{}",
                render_text_output_with_mode(&result, ClinicianMode::Both)
            );
            let payload = serde_json::to_string_pretty(&result)
                .map_err(|err| format!("json serialization failed: {}", err))?;
            println!("\n---\n{}", payload);
        }
    }
    Ok(())
}

fn run_command(command: Command) -> Result<(), String> {
    match command {
        Command::Convert(args) => run_convert(args),
        Command::Batch(args) => run_batch(args),
        Command::Cloak(args) => run_cloak(args.command),
        Command::SelfCheck(_) => run_self_check(),
    }
}

fn run_convert(args: ConvertArgs) -> Result<(), String> {
    let style = PlaceholderStyle::from_str(&args.placeholder_style)?;
    let input = read_input(&args.input)?;
    if input.trim().is_empty() {
        return Err("input is empty".to_string());
    }

    let result = process_note(&input, style);
    let include_all = args.all || (!args.patient && !args.redact && args.clinician.is_none());
    let include_patient = include_all || args.patient;
    let include_clinician = include_all || args.clinician.is_some();
    let include_redact = include_all || args.redact;

    if let Some(out_dir) = args.out.as_ref() {
        let out_dir = Path::new(out_dir);
        if include_redact {
            write_string(&out_dir.join("protected.txt"), &result.protected_text)?;
        }
        if include_patient {
            write_string(
                &out_dir.join("patient_view.txt"),
                &format_patient_view(&result.patient_view),
            )?;
        }
        if include_clinician {
            let mode = parse_clinician_mode(args.clinician.as_deref());
            write_string(
                &out_dir.join("clinician_view.txt"),
                &format_clinician_view(&result.clinician_soap, &result.clinician_5cs, mode),
            )?;
        }
    }

    match args.format {
        OutputFormat::Json => {
            let payload = serde_json::to_string_pretty(&result)
                .map_err(|err| format!("json serialization failed: {}", err))?;
            println!("{}", payload);
        }
        OutputFormat::Text => {
            let mode = parse_clinician_mode(args.clinician.as_deref());
            println!("{}", render_text_output_with_mode(&result, mode));
        }
        OutputFormat::Both => {
            let mode = parse_clinician_mode(args.clinician.as_deref());
            println!("{}", render_text_output_with_mode(&result, mode));
            let payload = serde_json::to_string_pretty(&result)
                .map_err(|err| format!("json serialization failed: {}", err))?;
            println!("\n---\n{}", payload);
        }
    }

    Ok(())
}

fn run_batch(args: BatchArgs) -> Result<(), String> {
    let input_dir = Path::new(&args.input_dir);
    if !input_dir.exists() {
        return Err(format!(
            "input directory not found: {}",
            input_dir.display()
        ));
    }
    let out_dir = args.out.clone().unwrap_or_else(|| "batch_out".to_string());
    let out_dir = Path::new(&out_dir);

    let files = list_files(input_dir)?;
    let mut coverage_rows = Vec::new();
    for file in files {
        if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
            let matches = glob::Pattern::new(&args.glob)
                .map_err(|err| format!("invalid glob: {}", err))?
                .matches(name);
            if !matches {
                continue;
            }
        }
        let content = read_to_string(&file)?;
        let result = process_note(&content, PlaceholderStyle::Protected);
        let rel = file.strip_prefix(input_dir).unwrap_or(&file);
        let dest = out_dir.join(rel);
        write_string(&dest, &result.protected_text)?;
        coverage_rows.push((rel.display().to_string(), result.coverage));
    }
    if !coverage_rows.is_empty() {
        let csv_path = out_dir.join("batch_coverage.csv");
        let mut writer = csv::Writer::from_path(&csv_path)
            .map_err(|err| format!("failed to write csv: {}", err))?;
        writer
            .write_record([
                "file",
                "fields_found",
                "fields_missing",
                "names",
                "phones",
                "emails",
                "dobs",
                "ids",
                "addresses",
            ])
            .map_err(|err| format!("failed to write csv header: {}", err))?;
        for (file, report) in coverage_rows {
            writer
                .write_record([
                    file,
                    report.fields_found.to_string(),
                    report.fields_missing.join("|"),
                    report.protected_counts.names.to_string(),
                    report.protected_counts.phones.to_string(),
                    report.protected_counts.emails.to_string(),
                    report.protected_counts.dobs.to_string(),
                    report.protected_counts.ids.to_string(),
                    report.protected_counts.addresses.to_string(),
                ])
                .map_err(|err| format!("failed to write csv row: {}", err))?;
        }
        writer
            .flush()
            .map_err(|err| format!("failed to flush csv: {}", err))?;
    }
    println!("Batch complete. Outputs in {}", out_dir.display());
    Ok(())
}

fn run_cloak(command: CloakCommand) -> Result<(), String> {
    match command {
        CloakCommand::Scan(args) => {
            let config = notenest::cloak::io::load_config(args.config.as_deref())?;
            let engine = CloakEngine::new(config);
            let path = Path::new(&args.path);
            let files = list_files(path)?;
            let mut reports = Vec::new();
            for file in files {
                let content = read_to_string(&file)?;
                let result = engine.protect_file(&file.display().to_string(), &content);
                reports.push(result.report);
            }
            let aggregate = engine.aggregate_report(reports);
            let payload = serde_json::to_string_pretty(&aggregate)
                .map_err(|err| format!("failed to serialize report: {}", err))?;
            println!("{}", payload);
            if let Some(csv_path) = args.csv.as_ref() {
                let out_path = Path::new(csv_path);
                let mut writer = csv::Writer::from_path(out_path)
                    .map_err(|err| format!("failed to create csv: {}", err))?;
                writer
                    .write_record(["file", "phi_type", "count"])
                    .map_err(|err| format!("failed to write csv header: {}", err))?;
                for file in &aggregate.files {
                    for count in &file.counts {
                        writer
                            .write_record([&file.file, &count.phi_type, &count.count.to_string()])
                            .map_err(|err| format!("failed to write csv row: {}", err))?;
                    }
                }
                writer
                    .flush()
                    .map_err(|err| format!("failed to flush csv: {}", err))?;
            }
            let _ = args.col;
            Ok(())
        }
        CloakCommand::Protect(args) => {
            let config = notenest::cloak::io::load_config(args.config.as_deref())?;
            let engine = CloakEngine::new(config);
            let path = Path::new(&args.path);
            let out_dir = Path::new(&args.out);
            fs::create_dir_all(out_dir)
                .map_err(|err| format!("failed to create {}: {}", out_dir.display(), err))?;

            let mut reports = Vec::new();
            let mut mapping_entries: Vec<(String, String)> = Vec::new();
            let files = list_files(path)?;
            for file in files {
                let content = read_to_string(&file)?;
                let result = engine.protect_file(&file.display().to_string(), &content);
                let rel = file.strip_prefix(path).unwrap_or(&file);
                let dest = out_dir.join(rel);
                write_string(&dest, &result.protected_text)?;
                notenest::cloak::io::write_file_report(out_dir, &result.report)?;
                let report = result.report.clone();
                reports.push(report.clone());
                if args.emit_structured {
                    let structured =
                        process_note(&result.protected_text, PlaceholderStyle::Protected);
                    write_string(
                        &out_dir.join(format!("{}.patient.txt", rel.display())),
                        &format_patient_view(&structured.patient_view),
                    )?;
                    write_string(
                        &out_dir.join(format!("{}.clinician.txt", rel.display())),
                        &format_clinician_view(
                            &structured.clinician_soap,
                            &structured.clinician_5cs,
                            ClinicianMode::Both,
                        ),
                    )?;
                }
                if args.format == CloakOutputFormat::Text || args.format == CloakOutputFormat::Both
                {
                    let text_path = out_dir.join(format!("{}.protected.txt", rel.display()));
                    write_string(&text_path, &result.protected_text)?;
                }
                if args.format == CloakOutputFormat::Json || args.format == CloakOutputFormat::Both
                {
                    let json_path = out_dir.join(format!("{}.protected.json", rel.display()));
                    let payload = serde_json::to_string_pretty(&result.protected_text)
                        .map_err(|err| format!("failed to serialize json: {}", err))?;
                    write_string(&json_path, &payload)?;
                }
                mapping_entries.push((report.file.clone(), dest.display().to_string()));
            }
            let aggregate = engine.aggregate_report(reports);
            notenest::cloak::io::write_aggregate_report(out_dir, &aggregate)?;
            notenest::cloak::io::write_csv_report(out_dir, &aggregate)?;
            notenest::cloak::io::write_markdown_report(out_dir, &aggregate)?;
            if let Some(mapping_path) = args.mapping.as_ref() {
                let mapping_payload = serde_json::to_string_pretty(&mapping_entries)
                    .map_err(|err| format!("failed to serialize mapping: {}", err))?;
                write_string(Path::new(mapping_path), &mapping_payload)?;
            }
            let _ = args.mapping_pass;
            println!("Cloak protect complete. Outputs in {}", out_dir.display());
            Ok(())
        }
        CloakCommand::Config(args) => match args.command {
            CloakConfigCommand::Init { out } => {
                let path = out.unwrap_or_else(|| "notenest-cloak.yml".to_string());
                notenest::cloak::io::write_default_config(Path::new(&path))?;
                println!("Wrote config to {}", path);
                Ok(())
            }
            CloakConfigCommand::Validate { path } => {
                let config = notenest::cloak::io::load_config(path.as_deref())?;
                let payload = serde_json::to_string_pretty(&config)
                    .map_err(|err| format!("failed to serialize config: {}", err))?;
                println!("{}", payload);
                Ok(())
            }
        },
        CloakCommand::Report(args) => {
            let run_dir = Path::new(&args.run);
            let json_path = if run_dir.is_dir() {
                run_dir.join("cloak_run_report.json")
            } else {
                run_dir.to_path_buf()
            };
            let content = read_to_string(&json_path)?;
            let report: notenest::cloak::report::AggregateReport =
                serde_json::from_str(&content)
                    .map_err(|err| format!("failed to parse report: {}", err))?;
            let out_dir = args.out.unwrap_or_else(|| "report_out".to_string());
            let out_dir = Path::new(&out_dir);
            fs::create_dir_all(out_dir)
                .map_err(|err| format!("failed to create {}: {}", out_dir.display(), err))?;
            match args.format {
                ReportFormat::Csv => {
                    notenest::cloak::io::write_csv_report(out_dir, &report)?;
                }
                ReportFormat::Json => {
                    notenest::cloak::io::write_aggregate_report(out_dir, &report)?;
                }
                ReportFormat::Md => {
                    notenest::cloak::io::write_markdown_report(out_dir, &report)?;
                }
            }
            println!("Report written to {}", out_dir.display());
            Ok(())
        }
        CloakCommand::GitHook(args) => match args.command {
            notenest::cli::CloakGitHookCommand::Install {
                mode,
                config,
                paths,
            } => {
                let hook_dir = Path::new(".git/hooks");
                fs::create_dir_all(hook_dir)
                    .map_err(|err| format!("failed to create hooks dir: {}", err))?;
                let hook_path = hook_dir.join(mode);
                let config_arg = config.unwrap_or_else(|| "notenest-cloak.yml".to_string());
                let paths_arg = if paths.is_empty() {
                    ".".to_string()
                } else {
                    paths.join(" ")
                };
                let script = format!(
                    "#!/usr/bin/env bash\nnotenest cloak scan {} --config {}\n",
                    paths_arg, config_arg
                );
                write_string(&hook_path, &script)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&hook_path)
                        .map_err(|err| format!("failed to read hook: {}", err))?
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&hook_path, perms)
                        .map_err(|err| format!("failed to set hook perms: {}", err))?;
                }
                println!("Installed hook at {}", hook_path.display());
                Ok(())
            }
        },
    }
}

fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|err| format!("failed to read stdin: {}", err))?;
        return Ok(buffer);
    }

    fs::read_to_string(path).map_err(|err| format!("failed to read {}: {}", path, err))
}

fn format_patient_view(view: &notenest::PatientView) -> String {
    notenest::notenest::render_patient_view(view)
}

fn format_clinician_view(
    soap: &notenest::ClinicianSoap,
    five_cs: &notenest::Clinician5Cs,
    mode: ClinicianMode,
) -> String {
    notenest::notenest::render_clinician_view(soap, five_cs, mode)
}

fn parse_clinician_mode(value: Option<&str>) -> ClinicianMode {
    match value.unwrap_or("").to_lowercase().as_str() {
        "soap" => ClinicianMode::Soap,
        "5cs" | "5c" => ClinicianMode::FiveCs,
        _ => ClinicianMode::Both,
    }
}

fn run_self_check() -> Result<(), String> {
    let patient_json = include_str!("../fixtures/examples/patient_view.example.json");
    let soap_json = include_str!("../fixtures/examples/clinician_soap.example.json");

    let _: notenest::PatientView = serde_json::from_str(patient_json)
        .map_err(|err| format!("patient view fixture invalid: {}", err))?;
    let _: notenest::ClinicianSoap = serde_json::from_str(soap_json)
        .map_err(|err| format!("clinician soap fixture invalid: {}", err))?;

    let sample = "Chief Complaint: Chest pain x2d\nPlan: follow up.\nemail demo@example.com";
    let outputs = process_note(sample, PlaceholderStyle::Protected);
    if !outputs.protected_text.contains("[EMAIL_PROTECTED]") {
        return Err("self-check failed: placeholder missing".to_string());
    }
    println!("Self-check passed.");
    Ok(())
}
