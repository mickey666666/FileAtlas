use crate::error::{AppError, AppResult};
use crate::filter::FilterSet;
use crate::model::{ContentMatch, FileIndex, FileRecord};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct SearchEngine {
    index: FileIndex,
}

impl SearchEngine {
    pub fn new(index: FileIndex) -> Self {
        Self { index }
    }

    pub fn find_by_name<'a>(
        &'a self,
        query: &str,
        filters: &FilterSet,
        limit: Option<usize>,
    ) -> Vec<&'a FileRecord> {
        let matcher = NameMatcher::new(query, filters.case_sensitive());
        let mut records: Vec<&FileRecord> = self
            .index
            .records
            .iter()
            .filter(|record| filters.accepts(record))
            .filter(|record| matcher.matches(record))
            .collect();
        filters.sort_records(&mut records);
        apply_limit(records, limit)
    }

    pub fn list_files<'a>(
        &'a self,
        filters: &FilterSet,
        limit: Option<usize>,
    ) -> Vec<&'a FileRecord> {
        let mut records: Vec<&FileRecord> = self
            .index
            .records
            .iter()
            .filter(|record| filters.accepts(record))
            .collect();
        filters.sort_records(&mut records);
        apply_limit(records, limit)
    }

    pub fn search_content(
        &self,
        options: &ContentSearchOptions,
        filters: &FilterSet,
    ) -> AppResult<Vec<ContentMatch>> {
        if options.query.is_empty() {
            return Err(AppError::invalid_arg("grep query cannot be empty"));
        }
        let mut matches = Vec::new();
        let matcher = LineMatcher::new(&options.query, options.case_sensitive);
        for record in self
            .index
            .records
            .iter()
            .filter(|record| filters.accepts(record))
        {
            if !record.is_text {
                continue;
            }
            let file_matches = search_file(record, &matcher, options.context)?;
            for item in file_matches {
                matches.push(item);
                if options.limit.is_some_and(|limit| matches.len() >= limit) {
                    return Ok(matches);
                }
            }
        }
        Ok(matches)
    }
}

#[derive(Debug, Clone)]
pub struct ContentSearchOptions {
    pub query: String,
    pub context: usize,
    pub case_sensitive: bool,
    pub limit: Option<usize>,
}

pub trait Matcher<T: ?Sized> {
    fn matches(&self, value: &T) -> bool;
}

#[derive(Debug, Clone)]
pub struct NameMatcher {
    query: String,
    case_sensitive: bool,
}

impl NameMatcher {
    pub fn new(query: &str, case_sensitive: bool) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive,
        }
    }
}

impl Matcher<FileRecord> for NameMatcher {
    fn matches(&self, value: &FileRecord) -> bool {
        value.contains_name(&self.query, self.case_sensitive)
    }
}

#[derive(Debug, Clone)]
pub struct LineMatcher {
    query: String,
    case_sensitive: bool,
}

impl LineMatcher {
    pub fn new(query: &str, case_sensitive: bool) -> Self {
        Self {
            query: if case_sensitive {
                query.to_string()
            } else {
                query.to_ascii_lowercase()
            },
            case_sensitive,
        }
    }

    pub fn highlight(&self, line: &str) -> String {
        if self.case_sensitive {
            line.replace(&self.query, &format!("[{}]", self.query))
        } else {
            highlight_case_insensitive(line, &self.query)
        }
    }
}

impl Matcher<str> for LineMatcher {
    fn matches(&self, value: &str) -> bool {
        if self.case_sensitive {
            value.contains(&self.query)
        } else {
            value.to_ascii_lowercase().contains(&self.query)
        }
    }
}

fn search_file(
    record: &FileRecord,
    matcher: &LineMatcher,
    context: usize,
) -> AppResult<Vec<ContentMatch>> {
    let file = File::open(&record.path)
        .map_err(|source| AppError::io(source, Some(record.path.clone()), "open text file"))?;
    let reader = BufReader::new(file);
    let mut results: Vec<ContentMatch> = Vec::new();
    let mut previous: VecDeque<(usize, String)> = VecDeque::new();
    let mut pending_after: Vec<(usize, usize)> = Vec::new();

    for (index, line_result) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line_result
            .map_err(|source| AppError::io(source, Some(record.path.clone()), "read text line"))?;

        for (match_index, remaining) in pending_after.iter_mut() {
            if *remaining > 0 {
                if let Some(item) = results.get_mut(*match_index) {
                    item.after.push((line_number, line.clone()));
                }
                *remaining -= 1;
            }
        }
        pending_after.retain(|(_, remaining)| *remaining > 0);

        if matcher.matches(&line) {
            let mut item =
                ContentMatch::new(record.path.clone(), line_number, matcher.highlight(&line));
            item.before = previous.iter().cloned().collect();
            if context > 0 {
                pending_after.push((results.len(), context));
            }
            results.push(item);
        }

        if context > 0 {
            previous.push_back((line_number, line));
            while previous.len() > context {
                previous.pop_front();
            }
        }
    }
    Ok(results)
}

fn highlight_case_insensitive(line: &str, query_lower: &str) -> String {
    if query_lower.is_empty() {
        return line.to_string();
    }
    let line_lower = line.to_ascii_lowercase();
    let mut output = String::new();
    let mut byte_start = 0;
    let mut search_start = 0;
    while let Some(relative) = line_lower[search_start..].find(query_lower) {
        let start = search_start + relative;
        let end = start + query_lower.len();
        output.push_str(&line[byte_start..start]);
        output.push('[');
        output.push_str(&line[start..end]);
        output.push(']');
        byte_start = end;
        search_start = end;
    }
    output.push_str(&line[byte_start..]);
    output
}

fn apply_limit<T>(mut records: Vec<T>, limit: Option<usize>) -> Vec<T> {
    if let Some(limit) = limit {
        records.truncate(limit);
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FileRecord;
    use std::path::PathBuf;

    #[test]
    fn name_matcher_is_case_insensitive_by_default() {
        let record = FileRecord {
            path: PathBuf::from("Report.TXT"),
            name: "Report.TXT".to_string(),
            extension: Some("txt".to_string()),
            size: 10,
            modified: None,
            is_text: true,
            line_count: Some(1),
            checksum: 1,
        };
        let matcher = NameMatcher::new("report", false);
        assert!(matcher.matches(&record));
    }

    #[test]
    fn line_matcher_highlights_case_insensitive_text() {
        let matcher = LineMatcher::new("rust", false);
        assert_eq!(matcher.highlight("Rust is fast"), "[Rust] is fast");
    }

    #[test]
    fn line_matcher_respects_case_sensitivity() {
        let matcher = LineMatcher::new("rust", true);
        assert!(!matcher.matches("Rust"));
        assert!(matcher.matches("rust"));
    }
}
