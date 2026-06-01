use crate::config::ScanConfig;
use crate::error::{AppError, AppResult};
use crate::model::{FileIndex, FileRecord, SkippedEntry};
use crate::util::{checksum_bytes, make_absolute, normalize_extension};
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Scanner {
    config: ScanConfig,
}

impl Scanner {
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    pub fn scan(&self, root: &Path) -> AppResult<FileIndex> {
        let root = make_absolute(root)?;
        if !root.exists() {
            return Err(AppError::invalid_arg(format!(
                "scan root '{}' does not exist",
                root.display()
            )));
        }
        if !root.is_dir() {
            return Err(AppError::invalid_arg(format!(
                "scan root '{}' is not a directory",
                root.display()
            )));
        }
        let mut index = FileIndex::new(root.clone());
        self.scan_iterative(&root, &mut index)?;
        index
            .records
            .sort_by(|left, right| left.path.cmp(&right.path));
        Ok(index)
    }

    fn scan_iterative(&self, root: &Path, index: &mut FileIndex) -> AppResult<()> {
        let mut queue = VecDeque::new();
        queue.push_back(root.to_path_buf());
        while let Some(directory) = queue.pop_front() {
            let entries = match fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(source) => {
                    index.push_skipped(SkippedEntry::new(
                        directory,
                        format!("cannot read directory: {source}"),
                    ));
                    continue;
                }
            };
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(source) => {
                        index.push_skipped(SkippedEntry::new(
                            directory.clone(),
                            format!("cannot read directory entry: {source}"),
                        ));
                        continue;
                    }
                };
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if self.config.should_ignore_name(&file_name) {
                    index.push_skipped(SkippedEntry::new(path, "ignored by default rule"));
                    continue;
                }
                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(source) => {
                        index.push_skipped(SkippedEntry::new(
                            path,
                            format!("cannot read file type: {source}"),
                        ));
                        continue;
                    }
                };
                if file_type.is_symlink() && !self.config.follow_links {
                    index.push_skipped(SkippedEntry::new(path, "symbolic link skipped"));
                    continue;
                }
                if file_type.is_dir() {
                    queue.push_back(path);
                } else if file_type.is_file() {
                    match self.build_record(path.clone()) {
                        Ok(record) => index.push_record(record),
                        Err(err) => index.push_skipped(SkippedEntry::new(path, err.to_string())),
                    }
                }
            }
        }
        Ok(())
    }

    fn build_record(&self, path: PathBuf) -> AppResult<FileRecord> {
        let metadata = fs::metadata(&path)
            .map_err(|source| AppError::io(source, Some(path.clone()), "read metadata"))?;
        let size = metadata.len();
        let modified = metadata.modified().ok();
        let sample = self.read_sample(&path)?;
        let checksum = checksum_bytes(&sample);
        let is_text = self.detect_text(&path, &sample);
        let line_count = if is_text && size <= self.config.max_counted_text_size {
            self.count_lines(&path).ok()
        } else {
            None
        };
        Ok(FileRecord::new(
            path, size, modified, is_text, line_count, checksum,
        ))
    }

    fn read_sample(&self, path: &Path) -> AppResult<Vec<u8>> {
        let mut file = File::open(path)
            .map_err(|source| AppError::io(source, Some(path.to_path_buf()), "open file"))?;
        let mut buffer = vec![0; self.config.max_text_sample];
        let read = file
            .read(&mut buffer)
            .map_err(|source| AppError::io(source, Some(path.to_path_buf()), "read file sample"))?;
        buffer.truncate(read);
        Ok(buffer)
    }

    fn detect_text(&self, path: &Path, sample: &[u8]) -> bool {
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str())
            && self
                .config
                .is_known_text_extension(&normalize_extension(ext))
        {
            return true;
        }
        if sample.is_empty() {
            return true;
        }
        if sample.contains(&0) {
            return false;
        }
        std::str::from_utf8(sample).is_ok()
    }

    fn count_lines(&self, path: &Path) -> AppResult<usize> {
        let content = fs::read_to_string(path)
            .map_err(|source| AppError::io(source, Some(path.to_path_buf()), "read text file"))?;
        Ok(content.lines().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detect_text_accepts_utf8() {
        let scanner = Scanner::new(ScanConfig::default());
        assert!(scanner.detect_text(Path::new("note.unknown"), b"hello\nworld"));
    }

    #[test]
    fn detect_text_rejects_null_bytes() {
        let scanner = Scanner::new(ScanConfig::default());
        assert!(!scanner.detect_text(Path::new("data.bin"), b"abc\0def"));
    }

    #[test]
    fn scan_reads_basic_file() {
        let temp = make_test_dir("scan_reads_basic_file");
        let file = temp.join("hello.txt");
        let mut handle = File::create(&file).unwrap();
        writeln!(handle, "hello").unwrap();
        writeln!(handle, "world").unwrap();
        let scanner = Scanner::new(ScanConfig::default());
        let index = scanner.scan(&temp).unwrap();
        assert_eq!(index.records.len(), 1);
        assert_eq!(index.records[0].line_count, Some(2));
        let _ = fs::remove_file(file);
        let _ = fs::remove_dir(temp);
    }

    fn make_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rust_finder_{name}_{}", std::process::id()));
        let _ = fs::create_dir(&dir);
        dir
    }
}
