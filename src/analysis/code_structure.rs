use crate::error::{AppError, AppResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodeStructureReport {
    pub path: PathBuf,
    pub language: &'static str,
    pub total_lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub hints: Vec<StructureHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureHint {
    pub name: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Rust,
    Python,
    CLike,
    Java,
    JavaScript,
}

pub fn analyze_file(path: &Path) -> AppResult<CodeStructureReport> {
    let language = detect_language(path)?;
    let content = fs::read_to_string(path)
        .map_err(|source| AppError::io(source, path.to_path_buf(), "read source file"))?;
    Ok(analyze_content(path.to_path_buf(), language, &content))
}

fn detect_language(path: &Path) -> AppResult<Language> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "rs" => Ok(Language::Rust),
        "py" => Ok(Language::Python),
        "c" | "cpp" | "h" => Ok(Language::CLike),
        "java" => Ok(Language::Java),
        "js" | "ts" => Ok(Language::JavaScript),
        _ => Err(AppError::invalid_arg(format!(
            "unsupported source file '{}'; expected .rs, .py, .c, .cpp, .h, .java, .js or .ts",
            path.display()
        ))),
    }
}

fn analyze_content(path: PathBuf, language: Language, content: &str) -> CodeStructureReport {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let blank_lines = lines.iter().filter(|line| line.trim().is_empty()).count();
    let comment_lines = count_comment_lines(language, &lines);
    let hints = match language {
        Language::Rust => rust_hints(&lines),
        Language::Python => python_hints(&lines),
        Language::CLike => c_like_hints(&lines),
        Language::Java => java_hints(&lines),
        Language::JavaScript => javascript_hints(&lines),
    };
    CodeStructureReport {
        path,
        language: language.name(),
        total_lines,
        blank_lines,
        comment_lines,
        hints,
    }
}

impl Language {
    fn name(self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::CLike => "C/C++",
            Language::Java => "Java",
            Language::JavaScript => "JavaScript/TypeScript",
        }
    }
}

fn count_comment_lines(language: Language, lines: &[&str]) -> usize {
    let mut count = 0;
    let mut in_block = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_block {
            count += 1;
            if trimmed.contains("*/") {
                in_block = false;
            }
            continue;
        }
        let line_comment = match language {
            Language::Python => trimmed.starts_with('#'),
            Language::Rust | Language::CLike | Language::Java | Language::JavaScript => {
                trimmed.starts_with("//")
            }
        };
        if line_comment {
            count += 1;
            continue;
        }
        if language != Language::Python && trimmed.starts_with("/*") {
            count += 1;
            if !trimmed.contains("*/") {
                in_block = true;
            }
        }
    }
    count
}

fn rust_hints(lines: &[&str]) -> Vec<StructureHint> {
    vec![
        hint("fn", count_rust_keyword(lines, "fn")),
        hint("struct", count_rust_keyword(lines, "struct")),
        hint("enum", count_rust_keyword(lines, "enum")),
        hint("trait", count_rust_keyword(lines, "trait")),
        hint("impl", count_rust_keyword(lines, "impl")),
        hint("mod", count_rust_keyword(lines, "mod")),
        hint("use", count_rust_keyword(lines, "use")),
    ]
}

fn python_hints(lines: &[&str]) -> Vec<StructureHint> {
    vec![
        hint("def", count_prefix(lines, "def ")),
        hint("class", count_prefix(lines, "class ")),
        hint("import", count_python_imports(lines)),
    ]
}

fn c_like_hints(lines: &[&str]) -> Vec<StructureHint> {
    vec![
        hint("include", count_prefix(lines, "#include")),
        hint("struct", count_prefix(lines, "struct ")),
        hint("class", count_prefix(lines, "class ")),
        hint("enum", count_prefix(lines, "enum ")),
        hint("function-like lines", count_function_like(lines)),
    ]
}

fn java_hints(lines: &[&str]) -> Vec<StructureHint> {
    vec![
        hint("class", count_word(lines, "class")),
        hint("interface", count_word(lines, "interface")),
        hint("enum", count_word(lines, "enum")),
        hint("method-like lines", count_function_like(lines)),
    ]
}

fn javascript_hints(lines: &[&str]) -> Vec<StructureHint> {
    vec![
        hint("function", count_word(lines, "function")),
        hint("class", count_word(lines, "class")),
        hint("import", count_prefix(lines, "import ")),
        hint(
            "arrow functions",
            lines.iter().filter(|line| line.contains("=>")).count(),
        ),
    ]
}

fn hint(name: &'static str, count: usize) -> StructureHint {
    StructureHint { name, count }
}

fn count_rust_keyword(lines: &[&str], keyword: &str) -> usize {
    lines
        .iter()
        .filter(|line| contains_word(strip_inline_comment(line), keyword))
        .count()
}

fn count_prefix(lines: &[&str], prefix: &str) -> usize {
    lines
        .iter()
        .filter(|line| strip_inline_comment(line).trim_start().starts_with(prefix))
        .count()
}

fn count_word(lines: &[&str], word: &str) -> usize {
    lines
        .iter()
        .filter(|line| contains_word(strip_inline_comment(line), word))
        .count()
}

fn count_python_imports(lines: &[&str]) -> usize {
    lines
        .iter()
        .filter(|line| {
            let trimmed = strip_python_comment(line).trim_start();
            trimmed.starts_with("import ") || trimmed.starts_with("from ")
        })
        .count()
}

fn count_function_like(lines: &[&str]) -> usize {
    lines
        .iter()
        .filter(|line| {
            let trimmed = strip_inline_comment(line).trim();
            trimmed.contains('(')
                && trimmed.contains(')')
                && (trimmed.ends_with('{') || trimmed.ends_with(';'))
                && !is_control_line(trimmed)
        })
        .count()
}

fn is_control_line(line: &str) -> bool {
    ["if", "for", "while", "switch", "catch", "return"]
        .iter()
        .any(|word| line.starts_with(word))
}

fn strip_inline_comment(line: &str) -> &str {
    line.split_once("//").map(|(code, _)| code).unwrap_or(line)
}

fn strip_python_comment(line: &str) -> &str {
    line.split_once('#').map(|(code, _)| code).unwrap_or(line)
}

fn contains_word(line: &str, word: &str) -> bool {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|part| part == word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_structure() {
        let report = analyze_content(
            PathBuf::from("main.rs"),
            Language::Rust,
            "use std::fs;\nmod cli;\nstruct App;\nenum Mode { A }\ntrait Run {}\nimpl Run for App {}\nfn main() {}\n",
        );
        assert_eq!(report.language, "Rust");
        assert_eq!(report.hints[0].count, 1);
        assert_eq!(report.hints[1].count, 1);
        assert_eq!(report.hints[2].count, 1);
        assert_eq!(report.hints[3].count, 1);
        assert_eq!(report.hints[4].count, 1);
    }

    #[test]
    fn detects_python_structure() {
        let report = analyze_content(
            PathBuf::from("tool.py"),
            Language::Python,
            "# comment\nimport os\nfrom pathlib import Path\nclass Tool:\n    def run(self):\n        pass\n",
        );
        assert_eq!(report.comment_lines, 1);
        assert_eq!(report.hints[0].count, 1);
        assert_eq!(report.hints[1].count, 1);
        assert_eq!(report.hints[2].count, 2);
    }
}
