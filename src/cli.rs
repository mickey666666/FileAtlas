use crate::config::ScanConfig;
use crate::error::{AppError, AppResult};
use crate::model::{SortDirection, SortField};
use crate::util::{parse_size, parse_usize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Cli {
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Help,
    Scan(ScanOptions),
    Find(FindOptions),
    Grep(GrepOptions),
    List(ListOptions),
    Stats(StatsOptions),
    Tree(TreeOptions),
    Inspect(InspectOptions),
    Export(ExportOptions),
    Shell,
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub root: PathBuf,
    pub index_path: PathBuf,
    pub follow_links: bool,
}

#[derive(Debug, Clone)]
pub struct FindOptions {
    pub query: String,
    pub index_path: PathBuf,
    pub filters: QueryFilterOptions,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct GrepOptions {
    pub query: String,
    pub index_path: PathBuf,
    pub filters: QueryFilterOptions,
    pub context: usize,
    pub case_sensitive: bool,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ListOptions {
    pub index_path: PathBuf,
    pub filters: QueryFilterOptions,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct StatsOptions {
    pub index_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub index_path: PathBuf,
    pub root: Option<PathBuf>,
    pub depth: usize,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub struct InspectOptions {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: String,
    pub output: PathBuf,
    pub index_path: PathBuf,
    pub filters: QueryFilterOptions,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFilterOptions {
    pub path_query: Option<String>,
    pub extensions: Vec<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
    pub text_only: bool,
    pub binary_only: bool,
    pub case_sensitive: bool,
    pub sort_field: Option<SortField>,
    pub sort_direction: SortDirection,
}

impl Cli {
    pub fn parse(args: Vec<String>) -> AppResult<Self> {
        let mut parser = ArgParser::new(args);
        let _program = parser.next();
        let Some(command) = parser.next() else {
            return Ok(Self {
                command: Command::Help,
            });
        };
        let command = match command.as_str() {
            "help" | "--help" | "-h" => Command::Help,
            "scan" => Command::Scan(parse_scan(&mut parser)?),
            "find" => Command::Find(parse_find(&mut parser)?),
            "grep" => Command::Grep(parse_grep(&mut parser)?),
            "list" => Command::List(parse_list(&mut parser)?),
            "stats" => Command::Stats(parse_stats(&mut parser)?),
            "tree" => Command::Tree(parse_tree(&mut parser)?),
            "inspect" => Command::Inspect(parse_inspect(&mut parser)?),
            "export" => Command::Export(parse_export(&mut parser)?),
            "shell" => Command::Shell,
            other => {
                return Err(AppError::invalid_arg(format!(
                    "unknown command '{other}', run `rust_finder help`"
                )));
            }
        };
        if parser.has_more() {
            return Err(AppError::invalid_arg(format!(
                "unexpected argument '{}'",
                parser.peek().unwrap_or_default()
            )));
        }
        Ok(Self { command })
    }
}

fn parse_scan(parser: &mut ArgParser) -> AppResult<ScanOptions> {
    let mut root = None;
    let mut index_path = ScanConfig::default_index_path();
    let mut follow_links = false;
    while let Some(arg) = parser.next() {
        match arg.as_str() {
            "--index" => index_path = PathBuf::from(parser.value("--index")?),
            "--follow-links" => follow_links = true,
            value if value.starts_with('-') => {
                return Err(AppError::invalid_arg(format!(
                    "unknown scan option '{value}'"
                )));
            }
            value => {
                if root.is_some() {
                    return Err(AppError::invalid_arg("scan accepts only one root path"));
                }
                root = Some(PathBuf::from(value));
            }
        }
    }
    Ok(ScanOptions {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        index_path,
        follow_links,
    })
}

fn parse_find(parser: &mut ArgParser) -> AppResult<FindOptions> {
    let query = parser.next_positional("find requires a query, e.g. `rust_finder find report`")?;
    let mut index_path = ScanConfig::default_index_path();
    let mut filters = QueryFilterOptions::default();
    let mut limit = None;
    parse_query_options(parser, &mut index_path, &mut filters, &mut limit)?;
    Ok(FindOptions {
        query,
        index_path,
        filters,
        limit,
    })
}

fn parse_grep(parser: &mut ArgParser) -> AppResult<GrepOptions> {
    let query = parser
        .next_positional("grep requires a query, e.g. `rust_finder grep ownership --context 1`")?;
    let mut index_path = ScanConfig::default_index_path();
    let mut filters = QueryFilterOptions::default();
    let mut limit = None;
    let mut context = 0;
    let mut case_sensitive = false;
    while let Some(arg) = parser.next() {
        match arg.as_str() {
            "--index" => index_path = PathBuf::from(parser.value("--index")?),
            "--path" => filters.path_query = Some(parser.value("--path")?),
            "--ext" => filters.extensions.push(parser.value("--ext")?),
            "--min-size" => filters.min_size = Some(parse_size(&parser.value("--min-size")?)?),
            "--max-size" => filters.max_size = Some(parse_size(&parser.value("--max-size")?)?),
            "--modified-after" => filters.modified_after = Some(parser.value("--modified-after")?),
            "--modified-before" => {
                filters.modified_before = Some(parser.value("--modified-before")?)
            }
            "--text" => filters.text_only = true,
            "--binary" => filters.binary_only = true,
            "--case-sensitive" => {
                filters.case_sensitive = true;
                case_sensitive = true;
            }
            "--context" | "-C" => context = parse_usize(&parser.value("--context")?, "context")?,
            "--limit" => limit = Some(parse_usize(&parser.value("--limit")?, "limit")?),
            "--sort" => filters.sort_field = parse_sort_field(&parser.value("--sort")?)?,
            "--desc" => filters.sort_direction = SortDirection::Desc,
            "--asc" => filters.sort_direction = SortDirection::Asc,
            other => {
                return Err(AppError::invalid_arg(format!(
                    "unknown grep option '{other}'"
                )));
            }
        }
    }
    Ok(GrepOptions {
        query,
        index_path,
        filters,
        context,
        case_sensitive,
        limit,
    })
}

fn parse_list(parser: &mut ArgParser) -> AppResult<ListOptions> {
    let mut index_path = ScanConfig::default_index_path();
    let mut filters = QueryFilterOptions::default();
    let mut limit = None;
    parse_query_options(parser, &mut index_path, &mut filters, &mut limit)?;
    Ok(ListOptions {
        index_path,
        filters,
        limit,
    })
}

fn parse_stats(parser: &mut ArgParser) -> AppResult<StatsOptions> {
    let mut index_path = ScanConfig::default_index_path();
    while let Some(arg) = parser.next() {
        match arg.as_str() {
            "--index" => index_path = PathBuf::from(parser.value("--index")?),
            other => {
                return Err(AppError::invalid_arg(format!(
                    "unknown stats option '{other}'"
                )));
            }
        }
    }
    Ok(StatsOptions { index_path })
}

fn parse_tree(parser: &mut ArgParser) -> AppResult<TreeOptions> {
    let mut index_path = ScanConfig::default_index_path();
    let mut root = None;
    let mut depth = 4;
    let mut limit = 12;
    while let Some(arg) = parser.next() {
        match arg.as_str() {
            "--index" => index_path = PathBuf::from(parser.value("--index")?),
            "--depth" => depth = parse_usize(&parser.value("--depth")?, "depth")?,
            "--limit" => limit = parse_usize(&parser.value("--limit")?, "limit")?,
            value if value.starts_with('-') => {
                return Err(AppError::invalid_arg(format!(
                    "unknown tree option '{value}'"
                )));
            }
            value => {
                if root.is_some() {
                    return Err(AppError::invalid_arg("tree accepts only one path"));
                }
                root = Some(PathBuf::from(value));
            }
        }
    }
    Ok(TreeOptions {
        index_path,
        root,
        depth,
        limit,
    })
}

fn parse_inspect(parser: &mut ArgParser) -> AppResult<InspectOptions> {
    let path = PathBuf::from(parser.next_positional(
        "inspect requires a source file, e.g. `rust_finder inspect src/main.rs`",
    )?);
    Ok(InspectOptions { path })
}

fn parse_export(parser: &mut ArgParser) -> AppResult<ExportOptions> {
    let format = parser.next_positional("export requires a format: json or csv")?;
    let output = PathBuf::from(parser.next_positional("export requires an output path")?);
    let mut index_path = ScanConfig::default_index_path();
    let mut filters = QueryFilterOptions::default();
    let mut limit = None;
    parse_query_options(parser, &mut index_path, &mut filters, &mut limit)?;
    Ok(ExportOptions {
        format,
        output,
        index_path,
        filters,
        limit,
    })
}

fn parse_query_options(
    parser: &mut ArgParser,
    index_path: &mut PathBuf,
    filters: &mut QueryFilterOptions,
    limit: &mut Option<usize>,
) -> AppResult<()> {
    while let Some(arg) = parser.next() {
        match arg.as_str() {
            "--index" => *index_path = PathBuf::from(parser.value("--index")?),
            "--path" => filters.path_query = Some(parser.value("--path")?),
            "--ext" => filters.extensions.push(parser.value("--ext")?),
            "--min-size" => filters.min_size = Some(parse_size(&parser.value("--min-size")?)?),
            "--max-size" => filters.max_size = Some(parse_size(&parser.value("--max-size")?)?),
            "--modified-after" => filters.modified_after = Some(parser.value("--modified-after")?),
            "--modified-before" => {
                filters.modified_before = Some(parser.value("--modified-before")?)
            }
            "--text" => filters.text_only = true,
            "--binary" => filters.binary_only = true,
            "--case-sensitive" => filters.case_sensitive = true,
            "--limit" => *limit = Some(parse_usize(&parser.value("--limit")?, "limit")?),
            "--sort" => filters.sort_field = parse_sort_field(&parser.value("--sort")?)?,
            "--desc" => filters.sort_direction = SortDirection::Desc,
            "--asc" => filters.sort_direction = SortDirection::Asc,
            other => {
                return Err(AppError::invalid_arg(format!("unknown option '{other}'")));
            }
        }
    }
    Ok(())
}

fn parse_sort_field(value: &str) -> AppResult<Option<SortField>> {
    SortField::parse(value)
        .map(Some)
        .ok_or_else(|| AppError::invalid_arg(format!("unknown sort field '{value}'")))
}

#[derive(Debug, Clone)]
struct ArgParser {
    args: Vec<String>,
    position: usize,
}

impl ArgParser {
    fn new(args: Vec<String>) -> Self {
        Self { args, position: 0 }
    }

    fn next(&mut self) -> Option<String> {
        let arg = self.args.get(self.position).cloned();
        if arg.is_some() {
            self.position += 1;
        }
        arg
    }

    fn peek(&self) -> Option<String> {
        self.args.get(self.position).cloned()
    }

    fn value(&mut self, option: &'static str) -> AppResult<String> {
        match self.next() {
            Some(value) if !value.starts_with("--") => Ok(value),
            Some(value) => Err(AppError::invalid_arg(format!(
                "{option} requires a value, got '{value}'"
            ))),
            None => Err(AppError::invalid_arg(format!("{option} requires a value"))),
        }
    }

    fn next_positional(&mut self, message: &'static str) -> AppResult<String> {
        match self.next() {
            Some(value) if !value.starts_with('-') => Ok(value),
            Some(value) => Err(AppError::invalid_arg(format!("{message}; got '{value}'"))),
            None => Err(AppError::invalid_arg(message)),
        }
    }

    fn has_more(&self) -> bool {
        self.position < self.args.len()
    }
}
