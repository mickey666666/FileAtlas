use crate::error::{AppError, AppResult};
use crate::model::FileRecord;
use crate::util::human_size;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn export_records(records: &[&FileRecord], format: &str, output: &Path) -> AppResult<()> {
    match format {
        "json" => export_json(records, output),
        "csv" => export_csv(records, output),
        other => Err(AppError::UnsupportedExportFormat(other.to_string())),
    }
}

fn export_json(records: &[&FileRecord], output: &Path) -> AppResult<()> {
    let file = File::create(output)
        .map_err(|source| AppError::io(source, Some(output.to_path_buf()), "create export file"))?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "[")?;
    for (index, record) in records.iter().enumerate() {
        let comma = if index + 1 == records.len() { "" } else { "," };
        writeln!(writer, "  {{")?;
        writeln!(
            writer,
            "    \"path\": \"{}\",",
            json_escape(&record.path.display().to_string())
        )?;
        writeln!(writer, "    \"name\": \"{}\",", json_escape(&record.name))?;
        writeln!(
            writer,
            "    \"extension\": {},",
            record
                .extension
                .as_ref()
                .map(|ext| format!("\"{}\"", json_escape(ext)))
                .unwrap_or_else(|| "null".to_string())
        )?;
        writeln!(writer, "    \"size\": {},", record.size)?;
        writeln!(
            writer,
            "    \"human_size\": \"{}\",",
            human_size(record.size)
        )?;
        writeln!(
            writer,
            "    \"modified\": {},",
            record
                .modified
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string())
        )?;
        writeln!(writer, "    \"is_text\": {},", record.is_text)?;
        writeln!(
            writer,
            "    \"line_count\": {},",
            record
                .line_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string())
        )?;
        writeln!(writer, "    \"checksum\": {}", record.checksum)?;
        writeln!(writer, "  }}{comma}")?;
    }
    writeln!(writer, "]")?;
    writer
        .flush()
        .map_err(|source| AppError::io(source, Some(output.to_path_buf()), "flush export file"))?;
    Ok(())
}

fn export_csv(records: &[&FileRecord], output: &Path) -> AppResult<()> {
    let file = File::create(output)
        .map_err(|source| AppError::io(source, Some(output.to_path_buf()), "create export file"))?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "path,name,extension,size,human_size,modified,is_text,line_count,checksum"
    )?;
    for record in records {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}",
            csv_escape(&record.path.display().to_string()),
            csv_escape(&record.name),
            csv_escape(record.extension.as_deref().unwrap_or("")),
            record.size,
            csv_escape(&human_size(record.size)),
            record
                .modified
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record.is_text,
            record
                .line_count
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record.checksum
        )?;
    }
    writer
        .flush()
        .map_err(|source| AppError::io(source, Some(output.to_path_buf()), "flush export file"))?;
    Ok(())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escape_wraps_commas() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn json_escape_wraps_quotes() {
        assert_eq!(json_escape("a\"b"), "a\\\"b");
    }
}
