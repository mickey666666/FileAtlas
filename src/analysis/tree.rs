use crate::model::FileIndex;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub root: Option<PathBuf>,
    pub max_depth: usize,
    pub max_children: usize,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            root: None,
            max_depth: 4,
            max_children: 12,
        }
    }
}

pub fn build_tree(index: &FileIndex, options: &TreeOptions) -> Vec<String> {
    let mut root = TreeNode::default();
    for record in &index.records {
        let relative = record
            .path
            .strip_prefix(&index.root)
            .unwrap_or(&record.path);
        if let Some(base) = &options.root
            && !relative.starts_with(base)
        {
            continue;
        }
        insert_path(&mut root, relative);
    }

    let root_name = options
        .root
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| {
            index
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| index.root.to_str().unwrap_or("."))
                .to_string()
        });
    let mut lines = vec![root_name.to_string()];
    render_children(&root, "", options, 1, &mut lines);
    lines
}

fn insert_path(root: &mut TreeNode, path: &Path) {
    let mut current = root;
    let components: Vec<String> = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect();
    for (index, component) in components.iter().enumerate() {
        let child = current.children.entry(component.clone()).or_default();
        child.is_file = index + 1 == components.len();
        current = child;
    }
}

fn render_children(
    node: &TreeNode,
    prefix: &str,
    options: &TreeOptions,
    depth: usize,
    lines: &mut Vec<String>,
) {
    if depth > options.max_depth {
        return;
    }

    let total = node.children.len();
    let visible = total.min(options.max_children);
    for (index, (name, child)) in node.children.iter().take(visible).enumerate() {
        let is_last_visible = index + 1 == visible && total <= options.max_children;
        let branch = if is_last_visible { "`-- " } else { "|-- " };
        lines.push(format!("{prefix}{branch}{name}"));

        if !child.is_file {
            let next_prefix = if is_last_visible {
                format!("{prefix}    ")
            } else {
                format!("{prefix}|   ")
            };
            render_children(child, &next_prefix, options, depth + 1, lines);
        }
    }

    if total > options.max_children {
        lines.push(format!(
            "{prefix}`-- ... and {} more",
            total - options.max_children
        ));
    }
}

#[derive(Debug, Clone, Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FileIndex, FileRecord};
    use std::path::PathBuf;

    #[test]
    fn tree_contains_nested_path() {
        let mut index = FileIndex::new(PathBuf::from("."));
        index.push_record(FileRecord {
            path: PathBuf::from("src/main.rs"),
            name: "main.rs".to_string(),
            extension: Some("rs".to_string()),
            size: 1,
            modified: None,
            is_text: true,
            line_count: Some(1),
            checksum: 0,
        });
        let lines = build_tree(&index, &TreeOptions::default());
        assert!(lines.iter().any(|line| line.contains("src")));
        assert!(lines.iter().any(|line| line.contains("main.rs")));
    }
}
