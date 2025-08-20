use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Result, Context};
use tracing::{event, Level};

use crate::models::sshes::Model;


pub struct SshKeyService {
    authorized_keys_path: PathBuf,
}

impl SshKeyService {
    pub fn new(user_home: &str) -> Self {
        Self {
            authorized_keys_path: PathBuf::from(format!("{}/.ssh/authorized_keys", user_home)),
        }
    }

    /// Append a new key to authorized_keys
    pub fn add_key(&self, key: &Model) -> Result<()> {
        event!(Level::INFO, "something has happened!");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.authorized_keys_path)
            .with_context(|| format!("failed to open {:?}", self.authorized_keys_path))?;
        let modified_string = key.public_key.clone().unwrap_or_default() + "\n";
        let bytes = modified_string.as_bytes();
        file.write_all(bytes)
            .context("failed to write ssh key")?;
        Ok(())
    }

    /// Remove a key (by fingerprint or exact match)
    pub fn remove_key(&self, key: &Model) -> Result<()> {
        let contents = read_to_string(&self.authorized_keys_path)
            .with_context(|| "failed to read authorized_keys")?;

        let mut new_contents = String::new();
        for line in contents.lines() {
            if !(line.contains(&key.public_key.clone().unwrap_or_default()))
            {
                new_contents.push_str(line);
                new_contents.push('\n');
            }
        }

        std::fs::write(&self.authorized_keys_path, new_contents)
            .with_context(|| "failed to write updated authorized_keys")?;
        Ok(())
    }

    /// Update key = remove old one + add new one
    pub fn update_key(&self, old_key: &Model, new_key: &Model) -> Result<()> {
        self.remove_key(old_key)?;
        self.add_key(new_key)?;
        Ok(())
    }
}

