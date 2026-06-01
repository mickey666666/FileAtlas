mod analysis;
mod cli;
mod config;
mod core;
mod error;
mod model;
mod presentation;
mod query;
mod util;

use crate::cli::{Cli, Command};
use crate::config::AppConfig;
use crate::error::AppResult;
use crate::filter::FilterSet;
use crate::index::IndexStore;
use crate::output::{Printer, print_help};
use crate::scanner::Scanner;
use crate::search::{ContentSearchOptions, SearchEngine};
use std::env;
use std::io::{self, Write};

pub use analysis::{code_structure, stats, tree};
pub use core::{index, scanner};
pub use presentation::{export, output};
pub use query::{filter, search};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let cli = Cli::parse(env::args().collect())?;
    let config = AppConfig::default();
    let printer = Printer::new(config.color_output);

    match cli.command {
        Command::Shell => run_shell(&config, &printer),
        command => execute_command(command, &config, &printer),
    }
}

fn execute_command(command: Command, config: &AppConfig, printer: &Printer) -> AppResult<()> {
    match command {
        Command::Help => {
            print_help();
        }
        Command::Scan(options) => {
            let mut scan_config = config.scan.clone();
            scan_config.follow_links = options.follow_links;
            let scanner = Scanner::new(scan_config);
            let index = scanner.scan(&options.root)?;
            let store = IndexStore::new(options.index_path);
            store.save(&index)?;
            printer.scan_summary(&index);
        }
        Command::Find(options) => {
            let store = IndexStore::new(options.index_path);
            let index = store.load()?;
            let filters = FilterSet::from_query_options(&options.filters)?;
            let engine = SearchEngine::new(index);
            let results = engine.find_by_name(&options.query, &filters, options.limit);
            printer.file_results(&results);
        }
        Command::Grep(options) => {
            let store = IndexStore::new(options.index_path);
            let index = store.load()?;
            let filters = FilterSet::from_query_options(&options.filters)?;
            let engine = SearchEngine::new(index);
            let search_options = ContentSearchOptions {
                query: options.query,
                context: options.context,
                case_sensitive: options.case_sensitive,
                limit: options.limit,
            };
            let results = engine.search_content(&search_options, &filters)?;
            printer.content_results(&results);
        }
        Command::List(options) => {
            let store = IndexStore::new(options.index_path);
            let index = store.load()?;
            let filters = FilterSet::from_query_options(&options.filters)?;
            let engine = SearchEngine::new(index);
            let results = engine.list_files(&filters, options.limit);
            printer.file_results(&results);
        }
        Command::Stats(options) => {
            let store = IndexStore::new(options.index_path);
            let index = store.load()?;
            let report = stats::build_report(&index);
            printer.stats_report(&report);
        }
        Command::Tree(options) => {
            let store = IndexStore::new(options.index_path);
            let index = store.load()?;
            let tree_options = tree::TreeOptions {
                root: options.root,
                max_depth: options.depth,
                max_children: options.limit,
            };
            for line in tree::build_tree(&index, &tree_options) {
                println!("{line}");
            }
        }
        Command::Inspect(options) => {
            let report = code_structure::analyze_file(&options.path)?;
            printer.code_structure_report(&report);
        }
        Command::Export(options) => {
            let store = IndexStore::new(options.index_path);
            let index = store.load()?;
            let filters = FilterSet::from_query_options(&options.filters)?;
            let engine = SearchEngine::new(index);
            let records = engine.list_files(&filters, options.limit);
            export::export_records(&records, &options.format, &options.output)?;
            println!(
                "exported {} records to {}",
                records.len(),
                options.output.display()
            );
        }
        Command::Shell => {
            println!("already in shell mode");
        }
    }

    Ok(())
}

fn run_shell(config: &AppConfig, printer: &Printer) -> AppResult<()> {
    println!("RustFinder shell");
    println!("type one command after each # prompt; type `exit` to quit");
    println!();
    println!("examples:");
    print_shell_example("scan .", "scan current directory and build the index");
    print_shell_example("stats", "show index statistics");
    print_shell_example("tree", "show indexed directory tree");
    print_shell_example("tree src", "show tree under one folder");
    print_shell_example("inspect src/cli.rs", "inspect one source file");
    print_shell_example("find README", "search file names or paths");
    print_shell_example("grep Result --ext rs", "search text content in Rust files");
    print_shell_example(
        "grep Result --path src/cli.rs",
        "search text content in one file",
    );
    print_shell_example("list --ext rs --limit 5", "list the first 5 Rust files");
    print_shell_example(
        "export csv result.csv --ext rs",
        "export Rust file records to CSV",
    );
    print_shell_example("exit", "quit shell mode");
    println!();
    let stdin = io::stdin();
    loop {
        print!("# ");
        io::stdout()
            .flush()
            .map_err(|source| crate::error::AppError::io(source, None, "flush prompt"))?;

        let mut line = String::new();
        let bytes = stdin
            .read_line(&mut line)
            .map_err(|source| crate::error::AppError::io(source, None, "read shell input"))?;
        if bytes == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
            break;
        }

        match parse_shell_command(line).and_then(|args| Cli::parse(args).map(|cli| cli.command)) {
            Ok(Command::Shell) => println!("already in shell mode"),
            Ok(command) => {
                if let Err(err) = execute_command(command, config, printer) {
                    eprintln!("error: {err}");
                }
                println!();
                println!();
            }
            Err(err) => {
                eprintln!("error: {err}");
                println!();
                println!();
            }
        }
    }
    Ok(())
}

fn print_shell_example(command: &str, description: &str) {
    println!("{command:<32} {description}");
}

fn parse_shell_command(line: &str) -> AppResult<Vec<String>> {
    let mut args = vec!["rust_finder".to_string()];
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err(crate::error::AppError::invalid_arg("missing closing quote"));
    }
    if !current.is_empty() {
        args.push(current);
    }
    Ok(args)
}
