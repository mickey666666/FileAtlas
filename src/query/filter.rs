use crate::cli::QueryFilterOptions;
use crate::error::{AppError, AppResult};
use crate::model::{FileRecord, SortDirection, SortField};
use crate::util::{compare_option_u64, extension_matches, normalize_extension, parse_date};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct FilterSet {
    path_query: Option<String>,
    extensions: Vec<String>,
    min_size: Option<u64>,
    max_size: Option<u64>,
    modified_after: Option<u64>,
    modified_before: Option<u64>,
    text_only: bool,
    binary_only: bool,
    case_sensitive: bool,
    sort_field: Option<SortField>,
    sort_direction: SortDirection,
}

impl FilterSet {
    pub fn from_query_options(options: &QueryFilterOptions) -> AppResult<Self> {
        if options.text_only && options.binary_only {
            return Err(AppError::invalid_arg(
                "--text and --binary cannot be used together",
            ));
        }
        let modified_after = options
            .modified_after
            .as_deref()
            .map(parse_date)
            .transpose()?;
        let modified_before = options
            .modified_before
            .as_deref()
            .map(parse_date)
            .transpose()?;
        if let (Some(after), Some(before)) = (modified_after, modified_before)
            && after > before
        {
            return Err(AppError::invalid_arg(
                "--modified-after must not be later than --modified-before",
            ));
        }
        if let (Some(min), Some(max)) = (options.min_size, options.max_size)
            && min > max
        {
            return Err(AppError::invalid_arg(
                "--min-size must not be greater than --max-size",
            ));
        }
        Ok(Self {
            path_query: options.path_query.as_ref().map(|query| {
                if options.case_sensitive {
                    normalize_path_query(query)
                } else {
                    normalize_path_query(&query.to_ascii_lowercase())
                }
            }),
            extensions: options
                .extensions
                .iter()
                .map(|ext| normalize_extension(ext))
                .collect(),
            min_size: options.min_size,
            max_size: options.max_size,
            modified_after,
            modified_before,
            text_only: options.text_only,
            binary_only: options.binary_only,
            case_sensitive: options.case_sensitive,
            sort_field: options.sort_field,
            sort_direction: options.sort_direction,
        })
    }

    pub fn accepts(&self, record: &FileRecord) -> bool {
        self.accepts_path(record)
            && self.accepts_extension(record)
            && self.accepts_size(record)
            && self.accepts_modified(record)
            && self.accepts_kind(record)
    }

    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn sort_records(&self, records: &mut Vec<&FileRecord>) {
        if let Some(field) = self.sort_field {
            records.sort_by(|left, right| self.compare_records(left, right, field));
            if self.sort_direction == SortDirection::Desc {
                records.reverse();
            }
        }
    }

    fn accepts_extension(&self, record: &FileRecord) -> bool {
        self.extensions.is_empty()
            || self
                .extensions
                .iter()
                .any(|extension| extension_matches(&record.extension, extension))
    }

    fn accepts_path(&self, record: &FileRecord) -> bool {
        let Some(query) = &self.path_query else {
            return true;
        };
        let path = normalize_path_query(&record.path.to_string_lossy());
        if self.case_sensitive {
            path.contains(query)
        } else {
            path.to_ascii_lowercase().contains(query)
        }
    }

    fn accepts_size(&self, record: &FileRecord) -> bool {
        if let Some(min_size) = self.min_size
            && record.size < min_size
        {
            return false;
        }
        if let Some(max_size) = self.max_size
            && record.size > max_size
        {
            return false;
        }
        true
    }

    fn accepts_modified(&self, record: &FileRecord) -> bool {
        let Some(modified) = record.modified else {
            return self.modified_after.is_none() && self.modified_before.is_none();
        };
        if let Some(after) = self.modified_after
            && modified < after
        {
            return false;
        }
        if let Some(before) = self.modified_before
            && modified > before + 86_399
        {
            return false;
        }
        true
    }

    fn accepts_kind(&self, record: &FileRecord) -> bool {
        if self.text_only && !record.is_text {
            return false;
        }
        if self.binary_only && record.is_text {
            return false;
        }
        true
    }

    fn compare_records(&self, left: &FileRecord, right: &FileRecord, field: SortField) -> Ordering {
        match field {
            SortField::Path => left.path.cmp(&right.path),
            SortField::Name => left.name.cmp(&right.name),
            SortField::Size => left.size.cmp(&right.size),
            SortField::Modified => compare_option_u64(left.modified, right.modified),
            SortField::Extension => left.extension.cmp(&right.extension),
        }
        .then_with(|| left.path.cmp(&right.path))
    }
}

fn normalize_path_query(value: &str) -> String {
    value.replace('\\', "/")
}

impl Default for FilterSet {
    fn default() -> Self {
        Self {
            path_query: None,
            extensions: Vec::new(),
            min_size: None,
            max_size: None,
            modified_after: None,
            modified_before: None,
            text_only: false,
            binary_only: false,
            case_sensitive: false,
            sort_field: None,
            sort_direction: SortDirection::Asc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::QueryFilterOptions;
    use crate::model::FileRecord;
    use std::path::PathBuf;

    fn record(name: &str, extension: Option<&str>, size: u64, is_text: bool) -> FileRecord {
        FileRecord {
            path: PathBuf::from(name),
            name: name.to_string(),
            extension: extension.map(str::to_string),
            size,
            modified: Some(1_700_000_000),
            is_text,
            line_count: if is_text { Some(1) } else { None },
            checksum: 0,
        }
    }

    #[test]
    fn extension_filter_accepts_matching_extension() {
        let options = QueryFilterOptions {
            extensions: vec!["rs".to_string()],
            ..QueryFilterOptions::default()
        };
        let filters = FilterSet::from_query_options(&options).unwrap();
        assert!(filters.accepts(&record("main.rs", Some("rs"), 10, true)));
        assert!(!filters.accepts(&record("main.md", Some("md"), 10, true)));
    }

    #[test]
    fn size_filter_rejects_too_small_files() {
        let options = QueryFilterOptions {
            min_size: Some(100),
            ..QueryFilterOptions::default()
        };
        let filters = FilterSet::from_query_options(&options).unwrap();
        assert!(!filters.accepts(&record("tiny.txt", Some("txt"), 10, true)));
        assert!(filters.accepts(&record("large.txt", Some("txt"), 200, true)));
    }

    #[test]
    fn text_and_binary_are_mutually_exclusive() {
        let options = QueryFilterOptions {
            text_only: true,
            binary_only: true,
            ..QueryFilterOptions::default()
        };
        assert!(FilterSet::from_query_options(&options).is_err());
    }

    #[test]
    fn path_filter_accepts_matching_path() {
        let options = QueryFilterOptions {
            path_query: Some("src/main".to_string()),
            ..QueryFilterOptions::default()
        };
        let filters = FilterSet::from_query_options(&options).unwrap();
        assert!(filters.accepts(&record("src/main.rs", Some("rs"), 10, true)));
        assert!(!filters.accepts(&record("README.md", Some("md"), 10, true)));
    }
}
