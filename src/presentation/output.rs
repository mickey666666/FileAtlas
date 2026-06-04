use crate::code_structure::CodeStructureReport;
use crate::model::{ContentMatch, FileIndex, FileRecord};
use crate::stats::{StatsReport, human_average_size, human_total_size, top_extensions};
use crate::util::human_size;

const OUTPUT_WIDTH: usize = 92;
const FIELD_WIDTH: usize = 14;
const ITEM_FIELD_WIDTH: usize = 10;
const CONTENT_FIELD_WIDTH: usize = 10;

#[derive(Debug, Clone)]
pub struct Printer {
    color: bool,
}

impl Printer {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    pub fn scan_summary(&self, index: &FileIndex) {
        self.section("Scan completed");
        let rows = [
            ("root", index.root.display().to_string()),
            ("files", index.records.len().to_string()),
            ("text files", index.text_count().to_string()),
            ("binary files", index.binary_count().to_string()),
            ("total size", human_size(index.total_size())),
            ("skipped", index.skipped.len().to_string()),
        ];
        self.key_values(&rows, FIELD_WIDTH);
        if !index.skipped.is_empty() {
            println!();
            self.section("Skipped entries");
            for skipped in index.skipped.iter().take(10) {
                self.key_values(
                    &[
                        ("path", skipped.path.display().to_string()),
                        ("reason", skipped.reason.clone()),
                    ],
                    FIELD_WIDTH,
                );
                println!();
            }
            if index.skipped.len() > 10 {
                println!("... and {} more", index.skipped.len() - 10);
            }
        }
    }

    pub fn file_results(&self, records: &[&FileRecord]) {
        if records.is_empty() {
            println!("no files matched");
            return;
        }
        self.section(&format!("{} file(s) matched", records.len()));
        for record in records {
            self.file_record(record);
        }
    }

    pub fn content_results(&self, matches: &[ContentMatch]) {
        if matches.is_empty() {
            println!("no content matched");
            return;
        }
        self.section(&format!("{} match(es)", matches.len()));
        for item in matches {
            println!();
            self.item_values(
                &[
                    ("file", item.path.display().to_string()),
                    ("line", item.line_number.to_string()),
                ],
                CONTENT_FIELD_WIDTH,
            );
            for (line_number, line) in &item.before {
                println!("{line_number:>5}- {line}");
            }
            println!("{:>5}: {}", item.line_number, self.match_line(&item.line));
            for (line_number, line) in &item.after {
                println!("{line_number:>5}+ {line}");
            }
        }
    }

    pub fn stats_report(&self, report: &StatsReport) {
        self.section("Index statistics");
        let rows = [
            ("root", report.root.clone()),
            ("created at", report.created_at.clone()),
            ("files", report.total_files.to_string()),
            ("text files", report.text_files.to_string()),
            ("binary files", report.binary_files.to_string()),
            ("total size", human_total_size(report)),
            ("average size", human_average_size(report)),
            ("known lines", report.total_lines.to_string()),
            ("skipped entries", report.skipped_count.to_string()),
        ];
        self.key_values(&rows, FIELD_WIDTH);
        println!();
        self.section("Top extensions");
        for (extension, count) in top_extensions(report, 10) {
            println!("{extension:<12} {count:>6}");
        }
        if let Some(file) = &report.largest_file {
            println!();
            self.section("Largest file");
            self.key_values(
                &[
                    ("path", file.path.clone()),
                    ("size", human_size(file.size)),
                    ("modified", file.modified.clone()),
                ],
                FIELD_WIDTH,
            );
        }
        if let Some(file) = &report.newest_file {
            println!();
            self.section("Newest file");
            self.key_values(
                &[
                    ("path", file.path.clone()),
                    ("modified", file.modified.clone()),
                ],
                FIELD_WIDTH,
            );
        }
        if let Some(file) = &report.oldest_file {
            println!();
            self.section("Oldest file");
            self.key_values(
                &[
                    ("path", file.path.clone()),
                    ("modified", file.modified.clone()),
                ],
                FIELD_WIDTH,
            );
        }
    }

    pub fn code_structure_report(&self, report: &CodeStructureReport) {
        self.section("Code structure");
        self.key_values(
            &[
                ("file", report.path.display().to_string()),
                ("language", report.language.to_string()),
                ("lines", report.total_lines.to_string()),
                ("blank lines", report.blank_lines.to_string()),
                ("comment lines", report.comment_lines.to_string()),
            ],
            FIELD_WIDTH,
        );
        println!();
        self.section("Structure hints");
        for hint in &report.hints {
            print_label_value(hint.name, &hint.count.to_string(), FIELD_WIDTH);
        }
    }

    fn section(&self, value: &str) {
        println!("{}", self.heading(value));
    }

    fn heading(&self, value: &str) -> String {
        if self.color {
            format!("\x1b[1;36m{value}\x1b[0m")
        } else {
            value.to_string()
        }
    }

    fn path(&self, value: &str) -> String {
        if self.color {
            format!("\x1b[32m{value}\x1b[0m")
        } else {
            value.to_string()
        }
    }

    fn match_line(&self, value: &str) -> String {
        if !self.color {
            return value.to_string();
        }
        value.replace('[', "\x1b[1;31m").replace(']', "\x1b[0m")
    }

    fn file_record(&self, record: &FileRecord) {
        println!();
        println!("{}", self.path(&record.name));
        self.item_values(
            &[
                ("path", record.path.display().to_string()),
                ("size", human_size(record.size)),
                ("type", record.kind_display().to_string()),
                ("ext", record.extension_display().to_string()),
                ("lines", record_lines(record)),
                ("modified", record.modified_display()),
            ],
            ITEM_FIELD_WIDTH,
        );
    }

    fn key_values(&self, rows: &[(&str, String)], label_width: usize) {
        for (label, value) in rows {
            print_label_value(label, value, label_width);
        }
    }

    fn item_values(&self, rows: &[(&str, String)], label_width: usize) {
        for (label, value) in rows {
            print_label_value(label, value, label_width);
        }
    }
}

fn record_lines(record: &FileRecord) -> String {
    record
        .line_count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn print_label_value(label: &str, value: &str, label_width: usize) {
    let prefix = format!("{label:<label_width$} ");
    let available = OUTPUT_WIDTH.saturating_sub(prefix.chars().count()).max(16);
    let chunks = wrap_text(value, available);
    if chunks.is_empty() {
        println!("{prefix}");
        return;
    }
    println!("{prefix}{}", chunks[0]);
    let padding = " ".repeat(prefix.chars().count());
    for chunk in chunks.iter().skip(1) {
        println!("{padding}{chunk}");
    }
}

fn wrap_text(value: &str, width: usize) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    let width = width.max(8);
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in value.chars() {
        current.push(ch);
        if current.chars().count() >= width {
            chunks.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

pub fn print_help() {
    println!(
        r#"RustFinder - local file search and content indexing

USAGE:
  rust_finder scan [path] [--index file]
  rust_finder find <query> [filters]
  rust_finder grep <query> [filters] [--context n]
  rust_finder list [filters]
  rust_finder stats [--index file]
  rust_finder tree [path] [--depth n] [--limit n]
  rust_finder inspect <source-file>
  rust_finder export <json|csv> <output> [filters]
  rust_finder shell

COMMON FILTERS:
  --index <file>              Use a custom index file
  --path <text>               Keep files whose path contains text
  --ext <ext>                 Keep only one extension; can be repeated
  --min-size <size>           Minimum file size, e.g. 10kb
  --max-size <size>           Maximum file size, e.g. 2mb
  --modified-after <date>     Keep files modified on/after YYYY-MM-DD
  --modified-before <date>    Keep files modified on/before YYYY-MM-DD
  --text                      Keep text files only
  --binary                    Keep binary files only
  --sort <field>              path, name, size, modified, ext
  --asc | --desc              Sort direction
  --limit <n>                 Limit number of results
  --jobs <n>                  Grep only: search files with n worker threads

QUICK START:
  cargo run -- scan .
  cargo run -- stats
  cargo run -- tree
  cargo run -- find README
  cargo run -- grep Rust --text --context 1
  cargo run -- shell

SCAN EXAMPLES:
  rust_finder scan .
  rust_finder scan ./src
  rust_finder scan . --index my_index.rfidx
  rust_finder scan . --follow-links

FIND EXAMPLES:
  rust_finder find README
  rust_finder find main --ext rs
  rust_finder find src --sort modified --desc --limit 5
  rust_finder find README --case-sensitive

GREP EXAMPLES:
  rust_finder grep Rust --text
  rust_finder grep Result --ext rs
  rust_finder grep Result --path src/main.rs
  rust_finder grep ownership --text -C 2
  rust_finder grep Result --ext rs --jobs 4 --limit 10
  rust_finder grep FileRecord --ext rs --case-sensitive

LIST EXAMPLES:
  rust_finder list
  rust_finder list --ext rs
  rust_finder list --text
  rust_finder list --sort size --desc --limit 10

STATS EXAMPLES:
  rust_finder stats
  rust_finder stats --index my_index.rfidx

TREE EXAMPLES:
  rust_finder tree
  rust_finder tree src
  rust_finder tree --depth 3
  rust_finder tree --depth 4 --limit 8

INSPECT EXAMPLES:
  rust_finder inspect src/main.rs
  rust_finder inspect src/cli.rs

EXPORT EXAMPLES:
  rust_finder export csv result.csv
  rust_finder export json result.json
  rust_finder export csv files.csv --ext rs
  rust_finder export csv recent.csv --sort modified --desc --limit 10

SHELL EXAMPLES:
  rust_finder shell
  # scan .
  # stats
  # tree
  # tree src
  # inspect src/main.rs
  # exit

WHEN USING CARGO:
  Add `cargo run --` before the rust_finder command, for example:
  cargo run -- find README
  cargo run -- grep Rust --text
"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_short_text_keeps_one_line() {
        assert_eq!(wrap_text("abc", 5), vec!["abc"]);
    }

    #[test]
    fn wrap_long_text_splits_lines() {
        assert_eq!(wrap_text("abcdefghijkl", 8), vec!["abcdefgh", "ijkl"]);
    }

    #[test]
    fn wrap_empty_text_has_no_chunks() {
        assert!(wrap_text("", 3).is_empty());
    }
}
