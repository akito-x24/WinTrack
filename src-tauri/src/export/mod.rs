use crate::database::Database;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_data(
    db: &Database,
    format: &str,
    start_date: &str,
    end_date: &str,
    output_path: &str,
) -> Result<String> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let sessions = db.get_sessions_range(start_date, end_date)?;

    match format {
        "csv" => export_csv(&sessions, output_path),
        "json" => export_json(&sessions, output_path),
        _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
    }
}

fn export_csv(sessions: &[serde_json::Value], output_path: &str) -> Result<String> {
    let mut wtr = csv::Writer::from_path(output_path)
        .with_context(|| format!("Cannot write to {}", output_path))?;

    wtr.write_record(&[
        "app_name", "executable_path", "category", "window_title",
        "start_time", "end_time", "duration_seconds", "was_idle",
    ])?;

    for s in sessions {
        wtr.write_record(&[
            s["app_name"].as_str().unwrap_or(""),
            s["executable_path"].as_str().unwrap_or(""),
            s["category"].as_str().unwrap_or(""),
            s["window_title"].as_str().unwrap_or(""),
            s["start_time"].as_str().unwrap_or(""),
            s["end_time"].as_str().unwrap_or(""),
            &s["duration_seconds"].as_i64().unwrap_or(0).to_string(),
            if s["was_idle"].as_bool().unwrap_or(false) { "true" } else { "false" },
        ])?;
    }

    wtr.flush()?;
    Ok(output_path.to_string())
}

fn export_json(sessions: &[serde_json::Value], output_path: &str) -> Result<String> {
    let json_str = serde_json::to_string_pretty(sessions)?;
    let mut file = File::create(output_path)?;
    file.write_all(json_str.as_bytes())?;
    Ok(output_path.to_string())
}
