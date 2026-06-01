use crate::model::{FileIndex, FileRecord};
use crate::util::human_size;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct StatsReport {
    pub root: String,
    pub created_at: String,
    pub total_files: usize,
    pub text_files: usize,
    pub binary_files: usize,
    pub total_size: u64,
    pub extension_counts: BTreeMap<String, usize>,
    pub largest_file: Option<FileSummary>,
    pub newest_file: Option<FileSummary>,
    pub oldest_file: Option<FileSummary>,
    pub skipped_count: usize,
    pub average_file_size: u64,
    pub total_lines: usize,
}

#[derive(Debug, Clone)]
pub struct FileSummary {
    pub path: String,
    pub size: u64,
    pub modified: String,
}

pub fn build_report(index: &FileIndex) -> StatsReport {
    let total_size = index.total_size();
    let total_files = index.records.len();
    let average_file_size = if total_files == 0 {
        0
    } else {
        total_size / total_files as u64
    };
    let total_lines = index
        .records
        .iter()
        .filter_map(|record| record.line_count)
        .sum();
    StatsReport {
        root: index.root_display(),
        created_at: index.created_display(),
        total_files,
        text_files: index.text_count(),
        binary_files: index.binary_count(),
        total_size,
        extension_counts: index.extension_counts(),
        largest_file: index.largest_file().map(file_summary),
        newest_file: index.newest_file().map(file_summary),
        oldest_file: index.oldest_file().map(file_summary),
        skipped_count: index.skipped.len(),
        average_file_size,
        total_lines,
    }
}

pub fn top_extensions(report: &StatsReport, limit: usize) -> Vec<(&str, usize)> {
    let mut values: Vec<(&str, usize)> = report
        .extension_counts
        .iter()
        .map(|(extension, count)| (extension.as_str(), *count))
        .collect();
    values.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    values.truncate(limit);
    values
}

pub fn human_total_size(report: &StatsReport) -> String {
    human_size(report.total_size)
}

pub fn human_average_size(report: &StatsReport) -> String {
    human_size(report.average_file_size)
}

fn file_summary(record: &FileRecord) -> FileSummary {
    FileSummary {
        path: record.path.display().to_string(),
        size: record.size,
        modified: record.modified_display(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FileIndex;
    use std::path::PathBuf;

    #[test]
    fn empty_report_has_zero_counts() {
        let index = FileIndex::new(PathBuf::from("."));
        let report = build_report(&index);
        assert_eq!(report.total_files, 0);
        assert_eq!(report.average_file_size, 0);
    }
}
