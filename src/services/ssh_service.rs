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
    /// Constructs a new `SshKeyService` pointing to the given user's `~/.ssh/authorized_keys`.
    ///
    /// # Arguments
    /// * `user_home` – The path to the user's home directory (e.g., `"/home/username"`).
    ///
    /// # Returns
    /// A configured `SshKeyService` instance.
    pub fn new(user_home: &str) -> Self {
        Self {
            authorized_keys_path: PathBuf::from(format!("{}/.ssh/authorized_keys", user_home)),
        }
    }

    /// Appends a public SSH key to the `authorized_keys` file.
    ///
    /// This will create the file if it does not exist, then open it
    /// for appending and write the key (plus a newline).
    ///
    /// # Arguments
    /// * `key` – A `Model` containing at least the `public_key` string.
    ///
    /// # Errors
    /// Returns an `Err` if the file cannot be opened or written.
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

    /// Removes all occurrences of the specified public key from `authorized_keys`.
    ///
    /// Reads the entire file, filters out lines containing the key,
    /// and writes the filtered contents back.
    ///
    /// # Arguments
    /// * `key` – A `Model` containing the `public_key` to remove.
    ///
    /// # Errors
    /// Returns an `Err` if the file cannot be read or the updated content cannot be written.
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

    /// Updates an existing SSH key by removing the old entry and adding the new one.
    ///
    /// This is a convenience wrapper that calls `remove_key` then `add_key`.
    ///
    /// # Arguments
    /// * `old_key` – The `Model` containing the existing key to remove.
    /// * `new_key` – The `Model` containing the new key to add.
    ///
    /// # Errors
    /// Returns an `Err` if either removal or addition fails.
    pub fn update_key(&self, old_key: &Model, new_key: &Model) -> Result<()> {
        self.remove_key(old_key)?;
        self.add_key(new_key)?;
        Ok(())
    }
}

