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
        r#"FileAtlas - local file index, content search, and metadata analysis

USAGE WITH CARGO:
  cargo run -- scan [path] [--index file]
  cargo run -- find <query> [filters]
  cargo run -- grep <query> [filters] [grep options]
  cargo run -- list [filters]
  cargo run -- stats [--index file]
  cargo run -- tree [path] [--depth n] [--limit n]
  cargo run -- inspect <source-file>
  cargo run -- export <json|csv> <output> [filters]
  cargo run -- shell

COMMANDS:
  scan       Scan a directory and save a local index
  find       Search file names and paths
  grep       Search text file contents
  list       List indexed files with filters and sorting
  stats      Show index statistics
  tree       Show a directory tree from the index
  inspect    Show lightweight source-code structure hints
  export     Export indexed file records to CSV or JSON
  shell      Enter interactive mode
  help       Show this help message

FILTERS:
  --index <file>              Use a custom index file
  --path <text>               Keep files whose path contains text
  --ext <ext>                 Keep one extension; can be repeated
  --min-size <size>           Minimum file size, e.g. 10kb
  --max-size <size>           Maximum file size, e.g. 2mb
  --modified-after <date>     Keep files modified on/after YYYY-MM-DD
  --modified-before <date>    Keep files modified on/before YYYY-MM-DD
  --text                      Keep text files only
  --binary                    Keep binary files only
  --case-sensitive            Match text with exact case
  --sort <field>              Sort by path, name, size, modified, or ext
  --asc | --desc              Sort direction
  --limit <n>                 Limit number of results

GREP OPTIONS:
  --context <n>, -C <n>       Show n lines before and after each match
  --jobs <n>, -j <n>          Search with n worker threads

QUICK START WITH CARGO:
  cargo run -- scan .
  cargo run -- stats
  cargo run -- find README
  cargo run -- list --ext rs --sort size --desc --limit 5
  cargo run -- grep Result --ext rs --jobs 4 --limit 10
  cargo run -- tree src --depth 3
  cargo run -- inspect src/main.rs
  cargo run -- export csv rust_files.csv --ext rs
  cargo run -- shell

SCAN EXAMPLES:
  cargo run -- scan .
  cargo run -- scan ./src
  cargo run -- scan . --index my_index.rfidx
  cargo run -- scan . --follow-links

FIND EXAMPLES:
  cargo run -- find README
  cargo run -- find main --ext rs
  cargo run -- find src --sort modified --desc --limit 5
  cargo run -- find README --case-sensitive

GREP EXAMPLES:
  cargo run -- grep Rust --text
  cargo run -- grep Result --ext rs
  cargo run -- grep Result --path src/main.rs
  cargo run -- grep ownership --text -C 2
  cargo run -- grep Result --ext rs --jobs 4 --limit 10
  cargo run -- grep FileRecord --ext rs --case-sensitive

LIST EXAMPLES:
  cargo run -- list
  cargo run -- list --ext rs
  cargo run -- list --text
  cargo run -- list --sort size --desc --limit 10

STATS EXAMPLES:
  cargo run -- stats
  cargo run -- stats --index my_index.rfidx

TREE EXAMPLES:
  cargo run -- tree
  cargo run -- tree src
  cargo run -- tree --depth 3
  cargo run -- tree --depth 4 --limit 8

INSPECT EXAMPLES:
  cargo run -- inspect src/main.rs
  cargo run -- inspect src/cli.rs

EXPORT EXAMPLES:
  cargo run -- export csv result.csv
  cargo run -- export json result.json
  cargo run -- export csv files.csv --ext rs
  cargo run -- export csv recent.csv --sort modified --desc --limit 10

SHELL MODE:
  Start shell mode with:
  cargo run -- shell

  After entering shell mode, type commands directly.
  The leading # below is the shell prompt; do not type it.

  # scan .
  # stats
  # find README
  # list --ext rs --sort size --desc --limit 5
  # grep Result --ext rs --jobs 4 --limit 10
  # tree src --depth 3
  # inspect src/main.rs
  # export csv rust_files.csv --ext rs
  # exit

DIRECT BINARY USAGE:
  After cargo build, you can also run:
  target\debug\rust_finder.exe help
  target\debug\rust_finder.exe scan .
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
