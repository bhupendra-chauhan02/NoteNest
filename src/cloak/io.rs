use std::path::{Path, PathBuf};

use crate::cloak::config::CloakConfig;
use crate::cloak::report::{AggregateReport, FileReport};
use crate::util::fs::{read_to_string, write_string};

pub fn load_config(path: Option<&str>) -> Result<CloakConfig, String> {
    match path {
        Some(path) => {
            let content = read_to_string(Path::new(path))?;
            serde_yaml::from_str(&content)
                .map_err(|err| format!("failed to parse config {}: {}", path, err))
        }
        None => Ok(CloakConfig::default()),
    }
}

pub fn write_default_config(path: &Path) -> Result<(), String> {
    let config = CloakConfig::default();
    let content = serde_yaml::to_string(&config)
        .map_err(|err| format!("failed to render config: {}", err))?;
    write_string(path, &content)
}

pub fn write_file_report(out_dir: &Path, report: &FileReport) -> Result<PathBuf, String> {
    let filename = format!("{}.cloak.json", report.file.replace(['/', '\\'], "_"));
    let path = out_dir.join(filename);
    let content = serde_json::to_string_pretty(report)
        .map_err(|err| format!("failed to serialize report: {}", err))?;
    write_string(&path, &content)?;
    Ok(path)
}

pub fn write_aggregate_report(out_dir: &Path, report: &AggregateReport) -> Result<PathBuf, String> {
    let path = out_dir.join("cloak_run_report.json");
    let content = serde_json::to_string_pretty(report)
        .map_err(|err| format!("failed to serialize report: {}", err))?;
    write_string(&path, &content)?;
    Ok(path)
}

pub fn write_csv_report(out_dir: &Path, report: &AggregateReport) -> Result<PathBuf, String> {
    let path = out_dir.join("cloak_run_report.csv");
    let mut writer =
        csv::Writer::from_path(&path).map_err(|err| format!("failed to create csv: {}", err))?;
    writer
        .write_record(["file", "phi_type", "count"])
        .map_err(|err| format!("failed to write csv header: {}", err))?;
    for file in &report.files {
        for count in &file.counts {
            writer
                .write_record([&file.file, &count.phi_type, &count.count.to_string()])
                .map_err(|err| format!("failed to write csv row: {}", err))?;
        }
    }
    writer
        .flush()
        .map_err(|err| format!("failed to flush csv: {}", err))?;
    Ok(path)
}

pub fn write_markdown_report(out_dir: &Path, report: &AggregateReport) -> Result<PathBuf, String> {
    let path = out_dir.join("cloak_run_report.md");
    let mut lines = Vec::new();
    lines.push("# Cloak Run Report".to_string());
    for file in &report.files {
        lines.push(format!("## {}", file.file));
        if file.counts.is_empty() {
            lines.push("- No PHI detected".to_string());
        } else {
            for count in &file.counts {
                lines.push(format!("- {}: {}", count.phi_type, count.count));
            }
        }
        if !file.flags.is_empty() {
            lines.push(format!("- Flags: {}", file.flags.join(", ")));
        }
    }
    write_string(&path, &lines.join("\n"))?;
    Ok(path)
}
