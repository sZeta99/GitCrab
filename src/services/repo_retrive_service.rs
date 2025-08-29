
use axum::{
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io,
    path::{Path as StdPath, PathBuf},
};
use tracing::{error, info, warn};

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
    pub is_file: bool,
    pub size: Option<u64>,
    pub children: Vec<RepoStructure>,
    pub extension: Option<String>,
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

// Helper functions for repository operations
pub fn is_binary_file(path: &StdPath) -> Result<bool> {
    let content = fs::read(path)?;
    
    // Check for null bytes which typically indicate binary content
    Ok(content.contains(&0u8))
}

pub fn get_file_extension(path: &StdPath) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

pub fn calculate_directory_size(path: &StdPath) -> u64 {
    let mut total_size = 0u64;
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            } else if entry_path.is_dir() {
                total_size += calculate_directory_size(&entry_path);
            }
        }
    }
    
    total_size
}

pub fn should_ignore_path(path: &StdPath) -> bool {
    let ignored_dirs = [".git", ".svn", "node_modules", "target", ".vscode", ".idea"];
    let ignored_files = [".DS_Store", "Thumbs.db", ".gitignore"];
    
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        return ignored_dirs.contains(&name) || ignored_files.contains(&name) || name.starts_with('.');
    }
    
    false
}

pub fn read_repository_structure(path: &StdPath, base_path: &StdPath) -> Result<RepoStructure> {
    let metadata = fs::metadata(path)?;
    let relative_path = path.strip_prefix(base_path)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    
    let name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if metadata.is_file() {
        Ok(RepoStructure {
            name,
            path: relative_path,
            is_file: true,
            size: Some(metadata.len()),
            children: Vec::new(),
            extension: get_file_extension(path),
        })
    } else if metadata.is_dir() {
        let mut children = Vec::new();
        let mut total_size = 0u64;

        match fs::read_dir(path) {
            Ok(entries) => {
                let mut entries: Vec<_> = entries.collect();
                entries.sort_by(|a, b| {
                    let a_name = a.as_ref().map(|e| e.file_name()).unwrap_or_default();
                    let b_name = b.as_ref().map(|e| e.file_name()).unwrap_or_default();
                    a_name.cmp(&b_name)
                });

                for entry in entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        
                        if should_ignore_path(&entry_path) {
                            continue;
                        }

                        match read_repository_structure(&entry_path, base_path) {
                            Ok(child_structure) => {
                                if child_structure.is_file {
                                    total_size += child_structure.size.unwrap_or(0);
                                } else {
                                    total_size += calculate_directory_size(&entry_path);
                                }
                                children.push(child_structure);
                            }
                            Err(e) => {
                                warn!("Failed to read structure for {:?}: {}", entry_path, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read directory {:?}: {}", path, e);
            }
        }

        Ok(RepoStructure {
            name,
            path: relative_path,
            is_file: false,
            size: Some(total_size),
            children,
            extension: None,
        })
    } else {
        Err(Error::string("Path is neither file nor directory"))
    }
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
pub fn clone_bare_repo(bare_repo_path: &StdPath, worktree_path: &StdPath) -> Result<()> {

    let mut opts = CloneOptions::new();

    opts.bare(false); // Clone into a working directory

    Repository::clone(bare_repo_path.to_str().unwrap(), worktree_path.to_str().unwrap())
        .map_err(|e| Error::string(format!("Failed to clone repository: {}", e)))?;

    Ok(())

}

pub async fn file_content(
    State(ctx): State<AppContext>,
    Path((repo_id, file_path)): Path<(String, String)>,
) -> Result<Json<FileContentResponse>> {
    info!("Fetching file content for repo: {}, file: {}", repo_id, file_path);
    
    let repo_path = PathBuf::from(format!("./repositories/{}", repo_id));
    let full_file_path = repo_path.join(&file_path);
    
    // Security check: ensure the file is within the repository directory
    if !full_file_path.starts_with(&repo_path) {
        warn!("Attempted path traversal attack for file: {}", file_path);
        return Err(Error::BadRequest("Invalid file path".into()));
    }
    
    if !full_file_path.exists() {
        return Err(Error::NotFound);
    }
    
    if !full_file_path.is_file() {
        return Err(Error::BadRequest("Path is not a file".into()));
    }
    
    match fs::metadata(&full_file_path) {
        Ok(metadata) => {
            let file_size = metadata.len();
            
            // Check if file is too large (e.g., 10MB limit)
            if file_size > 10 * 1024 * 1024 {
                return Err(Error::BadRequest("File too large to display".into()));
            }
            
            let is_binary = is_binary_file(&full_file_path).unwrap_or(true);
            
            if is_binary {
                let response = FileContentResponse {
                    repo_id,
                    file_path,
                    content: "[Binary file - content not displayed]".to_string(),
                    size: file_size,
                    is_binary: true,
                };
                return Ok(Json(response));
            }
            
            match fs::read_to_string(&full_file_path) {
                Ok(content) => {
                    let response = FileContentResponse {
                        repo_id,
                        file_path,
                        content,
                        size: file_size,
                        is_binary: false,
                    };
                    
                    info!("Successfully fetched file content");
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to read file content: {}", e);
                    Err(Error::InternalServerError)
                }
            }
        }
        Err(e) => {
            error!("Failed to get file metadata: {}", e);
            Err(Error::InternalServerError)
        }
    }
}

pub async fn list_repositories(
    State(ctx): State<AppContext>,
) -> Result<Json<Vec<String>>> {
    info!("Listing available repositories");
    
    let repos_dir = PathBuf::from("./repositories");
    
    if !repos_dir.exists() {
        info!("Repositories directory does not exist, creating it");
        if let Err(e) = fs::create_dir_all(&repos_dir) {
            error!("Failed to create repositories directory: {}", e);
            return Err(Error::InternalServerError);
        }
        return Ok(Json(Vec::new()));
    }
    
    match fs::read_dir(&repos_dir) {
        Ok(entries) => {
            let mut repositories = Vec::new();
            
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            repositories.push(name.to_string());
                        }
                    }
                }
            }
            
            repositories.sort();
            info!("Found {} repositories", repositories.len());
            Ok(Json(repositories))
        }
        Err(e) => {
            error!("Failed to read repositories directory: {}", e);
            Err(Error::InternalServerError)
        }
    }
}



