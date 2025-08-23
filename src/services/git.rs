
use std::path::PathBuf;
use std::fs::{self, create_dir_all};
use std::process::Command;
use tracing::{error, info, warn, debug};
use thiserror::Error;

/// Represents a custom error for GitService operations.
#[derive(Debug, Error)]
pub enum GitServiceError {
    #[error("Filesystem operation failed: {0}")]
    FilesystemError(String),
    #[error("Git command execution failed: {0}")]
    GitError(String),
    #[error("Invalid repository name: {0}")]
    InvalidRepositoryName(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// A service for managing Git repositories on the filesystem.
pub struct GitService {
    base_path: PathBuf,
    user: String
}

impl GitService {
    /// Create a new GitService instance with a base path and user.
    pub fn new(base_path: PathBuf, user: &str) -> Self {
        Self { base_path, user: user.to_string()}
    }

    /// Centralized method to build the repository path from its name.
    fn get_repository_path(&self, name: &str) -> Result<PathBuf, GitServiceError> {
        if name.trim().is_empty() {
            return Err(GitServiceError::InvalidRepositoryName(
                "Repository name cannot be empty".into(),
            ));
        }

        // Basic validation for repository name.
        let sanitized_name = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>();

        if sanitized_name != name {
            return Err(GitServiceError::InvalidRepositoryName(
                "Repository name contains invalid characters".into(),
            ));
        }

        Ok(self.base_path.join(format!("{}.git", sanitized_name)))
    }

    /// Create a bare Git repository.
    pub async fn create_bare_repository(&self, name: &str) -> Result<PathBuf, GitServiceError> {
        let repo_path = self.get_repository_path(name)?;

        // Check if the repository already exists.
        if repo_path.exists() {
            warn!("Repository '{}' already exists", name);
            return Err(GitServiceError::FilesystemError(format!(
                "Repository '{}' already exists at path: {:?}",
                name, repo_path
            )));
        }

        let mut rollback_steps = vec![];
        debug!("Creating bare git repository at {:?}", repo_path);

        // Ensure the parent directory exists.
        match create_dir_all(&repo_path.parent().unwrap()) {
            Ok(_) => rollback_steps.push(format!("delete-parent:{:?}", repo_path.parent())),
            Err(e) => {
                error!("Failed to create parent directory: {:?}", e);
                return Err(GitServiceError::FilesystemError(format!(
                    "Failed to create parent directory: {:?}",
                    e
                )));
            }
        }

        // Run the 'git init --bare' command.
        let git_init_result = tokio::process::Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&repo_path)
            .output()
            .await;
        // Set proper ownership
        let chown_output = match Command::new("chown")
                    .args(["-R", &format!("{}:{}", &self.user, &self.user)])
                    .arg(&repo_path)
                    .output() {
            Ok(it) => it,
            Err(e) => return Err(GitServiceError::FilesystemError(format!(
                    "Failed to set ownership: {:?}",
                    e
                ))),
        };

        if !chown_output.status.success() {
            warn!("Failed to set repository ownership: {}", 
                  String::from_utf8_lossy(&chown_output.stderr));
        }

        match git_init_result {
            Ok(output) if output.status.success() => {
                info!("Successfully created bare repository at {:?}", repo_path);
                rollback_steps.push(format!("delete:{:?}", repo_path));
            }
            Ok(output) => {
                error!(
                    "Git init failed: {:?}",
                    String::from_utf8_lossy(&output.stderr)
                );
                self.rollback(rollback_steps).await;
                return Err(GitServiceError::GitError(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
            Err(e) => {
                error!("Git init command failed: {:?}", e);
                self.rollback(rollback_steps).await;
                return Err(GitServiceError::GitError(format!(
                    "Git init command failed: {:?}",
                    e
                )));
            }
        }

        Ok(repo_path)
    }

    /// Deletes a Git repository.
    pub async fn delete_repository(&self, name: &str) -> Result<(), GitServiceError> {
        let repo_path = self.get_repository_path(name)?;

        if !repo_path.exists() {
            warn!(
                "Attempted to delete a repository that does not exist: {:?}",
                repo_path
            );
            return Err(GitServiceError::FilesystemError(format!(
                "Repository does not exist: {:?}",
                repo_path
            )));
        }

        debug!("Deleting repository at {:?}", repo_path);

        match fs::remove_dir_all(&repo_path) {
            Ok(_) => {
                info!("Successfully deleted repository at {:?}", repo_path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete repository {:?}: {:?}", repo_path, e);
                Err(GitServiceError::FilesystemError(format!(
                    "Failed to delete repository {:?}: {:?}",
                    repo_path, e
                )))
            }
        }
    }

    /// Move/Rename a Git repository.
    pub async fn rename_repository(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), GitServiceError> {
        let old_path = self.get_repository_path(old_name)?;
        let new_path = self.get_repository_path(new_name)?;

        // Ensure the source exists.
        if !old_path.exists() {
            return Err(GitServiceError::FilesystemError(format!(
                "Source repository does not exist: {:?}",
                old_path
            )));
        }

        // Ensure the target does not already exist.
        if new_path.exists() {
            return Err(GitServiceError::FilesystemError(format!(
                "Target repository already exists: {:?}",
                new_path
            )));
        }

        debug!(
            "Renaming repository from {:?} to {:?}",
            old_path, new_path
        );

        match tokio::fs::rename(&old_path, &new_path).await {
            Ok(_) => {
                info!(
                    "Successfully renamed repository from {:?} to {:?}",
                    old_path, new_path
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to rename repository from {:?} to {:?}: {:?}",
                    old_path, new_path, e
                );
                Err(GitServiceError::FilesystemError(format!(
                    "Failed to rename repository: {:?}",
                    e
                )))
            }
        }
    }

    /// Rollback a sequence of operations.
    async fn rollback(&self, operations: Vec<String>) {
        for operation in operations.into_iter().rev() {
            let parts: Vec<&str> = operation.splitn(2, ':').collect();
            if parts.len() != 2 {
                error!("Invalid rollback operation: {}", operation);
                continue;
            }

            match parts[0] {
                "delete" => {
                    let path = PathBuf::from(parts[1]);
                    if let Err(e) = tokio::fs::remove_dir_all(&path).await {
                        warn!("Failed to rollback delete operation for {:?}: {:?}", path, e);
                    }
                }
                "delete-parent" => {
                    let path = PathBuf::from(parts[1]);
                    if let Err(e) = tokio::fs::remove_dir_all(&path).await {
                        warn!(
                            "Failed to rollback delete-parent operation for {:?}: {:?}",
                            path, e
                        );
                    }
                }
                _ => error!("Unknown rollback operation: {}", operation),
            }
        }
    }
}


