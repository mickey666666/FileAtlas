use crate::error::{AppError, AppResult};
use crate::model::{FileIndex, FileRecord, SkippedEntry};
use crate::util::{
    escape_field, optional_u64_to_field, optional_usize_to_field, parse_bool, parse_optional_u64,
    parse_optional_usize, parse_u64, unescape_field,
};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const MAGIC: &str = "RUST_FINDER_INDEX_V1";

#[derive(Debug, Clone)]
pub struct IndexStore {
    path: PathBuf,
}

impl IndexStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn save(&self, index: &FileIndex) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                AppError::io(source, Some(parent.to_path_buf()), "create index directory")
            })?;
        }
        let file = File::create(&self.path)
            .map_err(|source| AppError::io(source, Some(self.path.clone()), "create index file"))?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{MAGIC}")?;
        writeln!(
            writer,
            "root\t{}",
            escape_field(&index.root.display().to_string())
        )?;
        writeln!(writer, "created_at\t{}", index.created_at)?;
        writeln!(writer, "records\t{}", index.records.len())?;
        for record in &index.records {
            writeln!(
                writer,
                "file\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                escape_path(&record.path),
                escape_field(&record.name),
                record
                    .extension
                    .as_deref()
                    .map(escape_field)
                    .unwrap_or_else(|| "-".to_string()),
                record.size,
                optional_u64_to_field(record.modified),
                record.is_text,
                optional_usize_to_field(record.line_count),
                record.checksum
            )?;
        }
        writeln!(writer, "skipped\t{}", index.skipped.len())?;
        for skipped in &index.skipped {
            writeln!(
                writer,
                "skip\t{}\t{}",
                escape_path(&skipped.path),
                escape_field(&skipped.reason)
            )?;
        }
        writer
            .flush()
            .map_err(|source| AppError::io(source, Some(self.path.clone()), "flush index file"))?;
        Ok(())
    }

    pub fn load(&self) -> AppResult<FileIndex> {
        if !self.path.exists() {
            return Err(AppError::MissingIndex(self.path.clone()));
        }
        let file = File::open(&self.path)
            .map_err(|source| AppError::io(source, Some(self.path.clone()), "open index file"))?;
        let mut lines = BufReader::new(file).lines();
        let magic = read_line(&mut lines, "missing magic header")?;
        if magic != MAGIC {
            return Err(AppError::InvalidIndex(format!(
                "unexpected header '{magic}'"
            )));
        }
        let root_line = read_line(&mut lines, "missing root line")?;
        let root = parse_prefixed_field(&root_line, "root")?;
        let created_line = read_line(&mut lines, "missing created_at line")?;
        let created_at = parse_prefixed_field(&created_line, "created_at")
            .and_then(|value| parse_u64(&value, "created_at"))?;
        let records_line = read_line(&mut lines, "missing records line")?;
        let record_count = parse_prefixed_field(&records_line, "records")
            .and_then(|value| parse_u64(&value, "records"))? as usize;

        let mut index = FileIndex {
            root: PathBuf::from(root),
            created_at,
            records: Vec::with_capacity(record_count),
            skipped: Vec::new(),
        };
        for _ in 0..record_count {
            let line = read_line(&mut lines, "missing file line")?;
            index.records.push(parse_record_line(&line)?);
        }
        let skipped_line = read_line(&mut lines, "missing skipped line")?;
        let skipped_count = parse_prefixed_field(&skipped_line, "skipped")
            .and_then(|value| parse_u64(&value, "skipped"))? as usize;
        for _ in 0..skipped_count {
            let line = read_line(&mut lines, "missing skip line")?;
            index.skipped.push(parse_skip_line(&line)?);
        }
        Ok(index)
    }
}

fn read_line<I>(lines: &mut I, message: &'static str) -> AppResult<String>
where
    I: Iterator<Item = Result<String, std::io::Error>>,
{
    match lines.next() {
        Some(Ok(line)) => Ok(line),
        Some(Err(source)) => Err(AppError::io(source, None, "read index line")),
        None => Err(AppError::InvalidIndex(message.to_string())),
    }
}

fn parse_prefixed_field(line: &str, prefix: &str) -> AppResult<String> {
    let mut parts = line.splitn(2, '\t');
    let actual = parts
        .next()
        .ok_or_else(|| AppError::InvalidIndex("empty index line".to_string()))?;
    if actual != prefix {
        return Err(AppError::InvalidIndex(format!(
            "expected prefix '{prefix}', got '{actual}'"
        )));
    }
    let value = parts
        .next()
        .ok_or_else(|| AppError::InvalidIndex(format!("missing value for '{prefix}'")))?;
    unescape_field(value)
}

fn parse_record_line(line: &str) -> AppResult<FileRecord> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() != 9 || parts[0] != "file" {
        return Err(AppError::InvalidIndex(format!(
            "invalid file line '{line}'"
        )));
    }
    let path = PathBuf::from(unescape_field(parts[1])?);
    let name = unescape_field(parts[2])?;
    let extension = if parts[3] == "-" {
        None
    } else {
        Some(unescape_field(parts[3])?)
    };
    let size = parse_u64(parts[4], "size")?;
    let modified = parse_optional_u64(parts[5])?;
    let is_text = parse_bool(parts[6])?;
    let line_count = parse_optional_usize(parts[7])?;
    let checksum = parse_u64(parts[8], "checksum")?;
    Ok(FileRecord {
        path,
        name,
        extension,
        size,
        modified,
        is_text,
        line_count,
        checksum,
    })
}

fn parse_skip_line(line: &str) -> AppResult<SkippedEntry> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() != 3 || parts[0] != "skip" {
        return Err(AppError::InvalidIndex(format!(
            "invalid skip line '{line}'"
        )));
    }
    Ok(SkippedEntry::new(
        PathBuf::from(unescape_field(parts[1])?),
        unescape_field(parts[2])?,
    ))
}

fn escape_path(path: &Path) -> String {
    escape_field(&path.display().to_string())
}
