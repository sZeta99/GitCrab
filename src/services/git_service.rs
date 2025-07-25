use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct GitService {
    pub repo_base_path: PathBuf,
    pub git_user: String,
}

impl GitService {
    pub fn new(repo_base_path: PathBuf, git_user: String) -> Self {
        Self {
            repo_base_path,
            git_user,
        }
    }

    pub async fn create_bare_repository(&self, name: &str) -> Result<PathBuf> {
        let repo_name = format!("{}.git", name);
        let repo_path = self.repo_base_path.join(&repo_name);

        // Check if repository already exists
        if repo_path.exists() {
            return Err(anyhow!("Repository '{}' already exists", name));
        }

        // Create parent directories if they don't exist
        if let Some(parent) = repo_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        info!("Creating bare repository: {}", repo_path.display());

        // Create bare repository
        let output = Command::new("git")
            .args(["init", "--bare"])
            .arg(&repo_path)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to create bare repository: {}", error_msg);
            return Err(anyhow!("Failed to create repository: {}", error_msg));
        }

        // Set proper ownership
        let chown_output = Command::new("chown")
            .args(["-R", &format!("{}:{}", self.git_user, self.git_user)])
            .arg(&repo_path)
            .output()?;

        if !chown_output.status.success() {
            warn!("Failed to set repository ownership: {}", 
                  String::from_utf8_lossy(&chown_output.stderr));
        }

        // Configure repository
        self.configure_repository(&repo_path).await?;

        info!("Successfully created repository: {}", repo_name);
        Ok(repo_path)
    }

    async fn configure_repository(&self, repo_path: &Path) -> Result<()> {
        // Enable receive.denyCurrentBranch = ignore for bare repos
        let config_output = Command::new("git")
            .args(["config", "receive.denyCurrentBranch", "ignore"])
            .current_dir(repo_path)
            .output()?;

        if !config_output.status.success() {
            warn!("Failed to configure repository: {}", 
                  String::from_utf8_lossy(&config_output.stderr));
        }

        // Set up hooks directory
        let hooks_dir = repo_path.join("hooks");
        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir).await?;
        }

        Ok(())
    }

    pub async fn delete_repository(&self, name: &str) -> Result<()> {
        let repo_name = format!("{}.git", name);
        let repo_path = self.repo_base_path.join(&repo_name);

        if !repo_path.exists() {
            return Err(anyhow!("Repository '{}' does not exist", name));
        }

        info!("Deleting repository: {}", repo_path.display());
        fs::remove_dir_all(&repo_path).await?;
        info!("Successfully deleted repository: {}", repo_name);

        Ok(())
    }

    pub async fn get_repository_size(&self, name: &str) -> Result<u64> {
        let repo_name = format!("{}.git", name);
        let repo_path = self.repo_base_path.join(&repo_name);

        if !repo_path.exists() {
            return Err(anyhow!("Repository '{}' does not exist", name));
        }

        let size = self.calculate_directory_size(&repo_path).await?;
        Ok(size)
    }

    async fn calculate_directory_size(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0u64;
        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                total_size += self.calculate_directory_size(&entry.path()).await?;
            } else {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    pub async fn list_repositories(&self) -> Result<Vec<String>> {
        let mut repositories = Vec::new();
        let mut entries = fs::read_dir(&self.repo_base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".git") {
                    if let Some(repo_name) = name.strip_suffix(".git") {
                        repositories.push(repo_name.to_string());
                    }
                }
            }
        }

        Ok(repositories)
    }
}
