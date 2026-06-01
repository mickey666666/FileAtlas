use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub color_output: bool,
    pub scan: ScanConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            color_output: true,
            scan: ScanConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub follow_links: bool,
    pub max_text_sample: usize,
    pub max_counted_text_size: u64,
    pub ignored_names: BTreeSet<String>,
    pub text_extensions: BTreeSet<String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        let ignored_names = [
            ".git",
            ".hg",
            ".svn",
            "target",
            "node_modules",
            ".rust_finder",
            "__pycache__",
            ".idea",
            ".vscode",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        let text_extensions = [
            "txt", "md", "rs", "toml", "json", "yaml", "yml", "csv", "tsv", "log", "html", "css",
            "js", "ts", "py", "java", "c", "cpp", "h", "hpp", "xml", "ini", "conf", "lock",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        Self {
            follow_links: false,
            max_text_sample: 8192,
            max_counted_text_size: 4 * 1024 * 1024,
            ignored_names,
            text_extensions,
        }
    }
}

impl ScanConfig {
    pub fn should_ignore_name(&self, name: &str) -> bool {
        self.ignored_names.contains(name)
    }

    pub fn is_known_text_extension(&self, ext: &str) -> bool {
        self.text_extensions.contains(&ext.to_ascii_lowercase())
    }

    pub fn default_index_path() -> PathBuf {
        PathBuf::from(".rust_finder").join("index.rfidx")
    }
}
