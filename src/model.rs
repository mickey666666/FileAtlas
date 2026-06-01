use crate::util::{display_system_time, normalize_extension};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRecord {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: Option<u64>,
    pub is_text: bool,
    pub line_count: Option<usize>,
    pub checksum: u64,
}

impl FileRecord {
    pub fn new(
        path: PathBuf,
        size: u64,
        modified: Option<SystemTime>,
        is_text: bool,
        line_count: Option<usize>,
        checksum: u64,
    ) -> Self {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(normalize_extension);
        let modified = modified.and_then(|time| {
            time.duration_since(UNIX_EPOCH)
                .ok()
                .map(|duration| duration.as_secs())
        });
        Self {
            path,
            name,
            extension,
            size,
            modified,
            is_text,
            line_count,
            checksum,
        }
    }

    pub fn modified_display(&self) -> String {
        self.modified
            .map(display_system_time)
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn extension_display(&self) -> &str {
        self.extension.as_deref().unwrap_or("-")
    }

    pub fn kind_display(&self) -> &'static str {
        if self.is_text { "text" } else { "binary" }
    }

    pub fn contains_name(&self, query: &str, case_sensitive: bool) -> bool {
        if case_sensitive {
            self.name.contains(query) || self.path.to_string_lossy().contains(query)
        } else {
            let q = query.to_ascii_lowercase();
            self.name.to_ascii_lowercase().contains(&q)
                || self
                    .path
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .contains(&q)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileIndex {
    pub root: PathBuf,
    pub created_at: u64,
    pub records: Vec<FileRecord>,
    pub skipped: Vec<SkippedEntry>,
}

impl FileIndex {
    pub fn new(root: PathBuf) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        Self {
            root,
            created_at,
            records: Vec::new(),
            skipped: Vec::new(),
        }
    }

    pub fn push_record(&mut self, record: FileRecord) {
        self.records.push(record);
    }

    pub fn push_skipped(&mut self, skipped: SkippedEntry) {
        self.skipped.push(skipped);
    }

    pub fn text_count(&self) -> usize {
        self.records.iter().filter(|record| record.is_text).count()
    }

    pub fn binary_count(&self) -> usize {
        self.records.len().saturating_sub(self.text_count())
    }

    pub fn total_size(&self) -> u64 {
        self.records.iter().map(|record| record.size).sum()
    }

    pub fn extension_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for record in &self.records {
            let key = record
                .extension
                .clone()
                .unwrap_or_else(|| "(none)".to_string());
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }

    pub fn largest_file(&self) -> Option<&FileRecord> {
        self.records.iter().max_by_key(|record| record.size)
    }

    pub fn newest_file(&self) -> Option<&FileRecord> {
        self.records
            .iter()
            .max_by_key(|record| record.modified.unwrap_or_default())
    }

    pub fn oldest_file(&self) -> Option<&FileRecord> {
        self.records
            .iter()
            .filter(|record| record.modified.is_some())
            .min_by_key(|record| record.modified.unwrap_or_default())
    }

    pub fn root_display(&self) -> String {
        self.root.display().to_string()
    }

    pub fn created_display(&self) -> String {
        display_system_time(self.created_at)
    }
}

#[derive(Debug, Clone)]
pub struct SkippedEntry {
    pub path: PathBuf,
    pub reason: String,
}

impl SkippedEntry {
    pub fn new(path: PathBuf, reason: impl Into<String>) -> Self {
        Self {
            path,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentMatch {
    pub path: PathBuf,
    pub line_number: usize,
    pub line: String,
    pub before: Vec<(usize, String)>,
    pub after: Vec<(usize, String)>,
}

impl ContentMatch {
    pub fn new(path: PathBuf, line_number: usize, line: String) -> Self {
        Self {
            path,
            line_number,
            line,
            before: Vec::new(),
            after: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Path,
    Name,
    Size,
    Modified,
    Extension,
}

impl SortField {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "path" => Some(SortField::Path),
            "name" => Some(SortField::Name),
            "size" => Some(SortField::Size),
            "modified" | "mtime" => Some(SortField::Modified),
            "ext" | "extension" => Some(SortField::Extension),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}
