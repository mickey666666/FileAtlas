use crate::error::{AppError, AppResult};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub fn normalize_extension(ext: &str) -> String {
    ext.trim_start_matches('.').to_ascii_lowercase()
}

pub fn parse_u64(value: &str, field: &'static str) -> AppResult<u64> {
    value.parse::<u64>().map_err(|source| AppError::ParseInt {
        source,
        value: value.to_string(),
        field,
    })
}

pub fn parse_usize(value: &str, field: &'static str) -> AppResult<usize> {
    value.parse::<usize>().map_err(|source| AppError::ParseInt {
        source,
        value: value.to_string(),
        field,
    })
}

pub fn parse_size(value: &str) -> AppResult<u64> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::invalid_arg("size value cannot be empty"));
    }
    let split = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(split);
    let base = parse_u64(number, "size")?;
    let multiplier = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024 * 1024,
        "g" | "gb" | "gib" => 1024 * 1024 * 1024,
        other => {
            return Err(AppError::invalid_arg(format!(
                "unknown size unit '{other}', expected b/kb/mb/gb"
            )));
        }
    };
    base.checked_mul(multiplier)
        .ok_or_else(|| AppError::invalid_arg("size value is too large"))
}

pub fn human_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let size_f = size as f64;
    if size_f >= GB {
        format!("{:.2} GB", size_f / GB)
    } else if size_f >= MB {
        format!("{:.2} MB", size_f / MB)
    } else if size_f >= KB {
        format!("{:.2} KB", size_f / KB)
    } else {
        format!("{size} B")
    }
}

pub fn parse_date(value: &str) -> AppResult<u64> {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidDate(value.to_string()));
    }
    let year = parse_i32(parts[0], value)?;
    let month = parse_i32(parts[1], value)?;
    let day = parse_i32(parts[2], value)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(AppError::InvalidDate(value.to_string()));
    }
    let days = days_from_civil(year, month as u32, day as u32);
    if days < 0 {
        return Err(AppError::InvalidDate(value.to_string()));
    }
    Ok((days as u64) * 86_400)
}

fn parse_i32(part: &str, original: &str) -> Result<i32, AppError> {
    part.parse::<i32>()
        .map_err(|_| AppError::InvalidDate(original.to_string()))
}

pub fn display_system_time(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let secs_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

pub fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe - 719468) as i64
}

pub fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    ((y + if m <= 2 { 1 } else { 0 }) as i32, m as u32, d as u32)
}

pub fn checksum_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn escape_field(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub fn unescape_field(value: &str) -> AppResult<String> {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }
        match chars.next() {
            Some('\\') => output.push('\\'),
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('t') => output.push('\t'),
            Some(other) => {
                return Err(AppError::InvalidIndex(format!(
                    "unknown escape sequence \\{other}"
                )));
            }
            None => {
                return Err(AppError::InvalidIndex(
                    "dangling escape sequence".to_string(),
                ));
            }
        }
    }
    Ok(output)
}

pub fn make_absolute(path: &Path) -> AppResult<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(|source| AppError::io(source, None, "read current directory"))
}

pub fn compare_option_u64(left: Option<u64>, right: Option<u64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

pub fn parse_bool(value: &str) -> AppResult<bool> {
    match value {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(AppError::InvalidIndex(format!("invalid bool '{value}'"))),
    }
}

pub fn parse_optional_u64(value: &str) -> AppResult<Option<u64>> {
    if value == "-" {
        Ok(None)
    } else {
        Ok(Some(parse_u64(value, "optional number")?))
    }
}

pub fn parse_optional_usize(value: &str) -> AppResult<Option<usize>> {
    if value == "-" {
        Ok(None)
    } else {
        Ok(Some(parse_usize(value, "optional number")?))
    }
}

pub fn optional_u64_to_field(value: Option<u64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

pub fn optional_usize_to_field(value: Option<usize>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

pub fn extension_matches(value: &Option<String>, query: &str) -> bool {
    value
        .as_ref()
        .map(|extension| extension == &normalize_extension(query))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_supports_units() {
        assert_eq!(parse_size("1kb").unwrap(), 1024);
        assert_eq!(parse_size("2mb").unwrap(), 2 * 1024 * 1024);
    }

    #[test]
    fn date_roundtrip_keeps_calendar_day() {
        let seconds = parse_date("2026-05-28").unwrap();
        assert!(display_system_time(seconds).starts_with("2026-05-28"));
    }

    #[test]
    fn escaped_fields_roundtrip_special_characters() {
        let original = "a\tb\nc\\d";
        let escaped = escape_field(original);
        assert_eq!(unescape_field(&escaped).unwrap(), original);
    }

    #[test]
    fn checksum_changes_when_bytes_change() {
        assert_ne!(checksum_bytes(b"abc"), checksum_bytes(b"abd"));
    }
}
