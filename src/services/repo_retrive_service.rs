
use git2::{Repository, ObjectType};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path as StdPath},
};

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct RepoParams {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct FileContentParams {
    pub repo_id: String,
    pub file_path: String,
}

#[derive(Debug, Serialize)]
pub struct RepoStructure {
    pub name: String,
    pub path: String,
    pub id: String,
    pub is_file: bool,
    pub size: Option<u64>,
    pub children: Vec<RepoStructure>,
    pub extension: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RepoResponse {
    pub id: String,
    pub name: String,
    pub structure: RepoStructure,
    pub total_files: usize,
    pub total_size: u64,
}

#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    pub repo_id: String,
    pub file_path: String,
    pub content: String,
    pub size: u64,
    pub is_binary: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub fn read_git_repository_structure(repo: &Repository) -> Result<RepoStructure> {

    // Get the HEAD commit
    let head = repo.head()
        .map_err(|e| -> Error {Error::string(&format!("Failed to get HEAD: {}", e))})?;

    let commit = head.peel_to_commit()
        .map_err(|e| -> Error {Error::string(&format!("Failed to get commit: {}", e))})?;
    
    let tree = commit.tree()
        .map_err(|e| -> Error {Error::string(&format!("Failed to get tree: {}", e))})?;

    // Build structure from the root tree
    read_git_tree_structure(repo, &tree, "", "")

}

pub fn read_git_tree_structure(
    repo: &Repository,
    tree: &git2::Tree,
    name: &str,
    path: &str,

) -> Result<RepoStructure> {

    let mut children = Vec::new();
    let mut total_size = 0u64;

    // Iterate through tree entries
    for entry in tree.iter() {
        let entry_name = entry.name().unwrap_or("");
        let entry_path = if path.is_empty() {
            entry_name.to_string()
        } else {
            format!("{}/{}", path, entry_name)
        };

        // Skip ignored files/directories
        if should_ignore_git_entry(entry_name) {
            continue;
        }

        match entry.kind() {
            Some(ObjectType::Tree) => {

                // It's a directory
                let subtree = entry.to_object(repo)
                    .map_err(|e| Error::string(&format!("Failed to get tree object: {}", e)))?
                    .peel_to_tree()
                    .map_err(|e| Error::string(&format!("Failed to peel to tree: {}", e)))?;
                let child_structure = read_git_tree_structure(repo, &subtree, entry_name, &entry_path)?;
                total_size += child_structure.size.unwrap_or(0);
                children.push(child_structure);
            }

            Some(ObjectType::Blob) => {
                // It's a file
                let blob = entry.to_object(repo)
                    .map_err(|e| Error::string(&format!("Failed to get blob object: {}", e)))?
                    .peel_to_blob()
                    .map_err(|e| Error::string(&format!("Failed to peel to blob: {}", e)))?;

                let file_size = blob.size() as u64;
                total_size += file_size;

                // Handle file content
                let is_binary = blob.is_binary();
                let file_content = if is_binary {
                    None // Skip binary files
                } else {
                    String::from_utf8(blob.content().to_vec()).ok()
                };
                let sanitized_id = path.replace("/", "_").replace("\\", "_");
                children.push(RepoStructure {
                    name: entry_name.to_string(),
                    path: entry_path,
                    id: sanitized_id,
                    is_file: true,
                    size: Some(file_size),
                    children: Vec::new(),
                    extension: get_file_extension_from_name(entry_name),
                    content: file_content,
                });
            }

            _ => {
                // Skip other object types (commits, tags)
                continue;
            }

        }

    }
    let sanitized_id = path.replace("/", "_").replace("\\", "_");

    // Sort children by name
    children.sort_by(|a, b| a.name.cmp(&b.name));
    let structure_name = if name.is_empty() { "root" } else { name };

    Ok(RepoStructure {
        name: structure_name.to_string(),
        path: path.to_string(),
        id: sanitized_id,
        is_file: false,
        size: Some(total_size),
        children,
        extension: None,
        content: None,

    })

}

pub fn should_ignore_git_entry(name: &str) -> bool {
    let ignored_dirs = ["node_modules", "target", ".vscode", ".idea"];
    let ignored_files = [".DS_Store", "Thumbs.db"];
    ignored_dirs.contains(&name) || ignored_files.contains(&name) || name.starts_with('.')
}

pub fn get_file_extension_from_name(name: &str) -> Option<String> {
    StdPath::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}
pub fn count_files_in_structure(structure: &RepoStructure) -> usize {
    if structure.is_file {
        1
    } else {
        structure.children.iter().map(count_files_in_structure).sum()
    }
}

pub fn get_total_size_from_structure(structure: &RepoStructure) -> u64 {
    structure.size.unwrap_or(0)
}

