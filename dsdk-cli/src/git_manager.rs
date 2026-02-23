// Copyright (c) 2026 Analog Devices, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::config::GitConfig;
use crate::git_operations;
use crate::messages;
use anyhow::Result;
use std::path::Path;

/// Returns true if the repo at `repo_path` has pending/changed files (uncommitted or staged).
///
/// # Errors
///
/// Returns an error if the git repository cannot be opened or the status cannot be determined.
pub fn repo_has_pending_changes(repo_path: &Path) -> Result<bool> {
    git_operations::is_repo_dirty(repo_path)
}

/// Updates/checks out the workspace repo at `repo_path` to the specified commit/tag.
///
/// # Errors
///
/// Returns an error if:
/// - The git repository cannot be opened
/// - The specified commit/tag cannot be found
/// - The checkout operation fails
pub fn update_workspace_repo(repo_path: &Path, commit: &str) -> Result<()> {
    let result = git_operations::checkout(repo_path, commit)?;

    if !result.is_success() {
        return Err(anyhow::anyhow!(
            "Failed to checkout {}: {}",
            commit,
            result.stderr
        ));
    }

    Ok(())
}

/// Updates a git repository by cloning or fetching from the configured URL.
///
/// # Errors
///
/// Returns an error if:
/// - The repository cannot be cloned or opened
/// - Network operations fail
/// - Git operations such as fetch or checkout fail
pub fn update_git(config: &GitConfig, mirror_path: &Path) -> Result<()> {
    let repo_path = mirror_path.join(&config.name);

    if repo_path.exists() {
        messages::status(&format!("Fetching {}...", config.name));

        let fetch_result = git_operations::fetch_all(&repo_path)?;

        if !fetch_result.is_success() {
            messages::error(&format!(
                "Failed to fetch {}: {}",
                config.name, fetch_result.stderr
            ));
            return Err(anyhow::anyhow!("Git fetch failed"));
        }
    } else {
        messages::status(&format!("Cloning {}...", config.name));

        let clone_result = git_operations::clone_repo(&config.url, &repo_path, None)?;

        if !clone_result.is_success() {
            messages::error(&format!(
                "Failed to clone {}: {}",
                config.name, clone_result.stderr
            ));
            return Err(anyhow::anyhow!("Git clone failed"));
        }
    }

    messages::status(&format!("Checking out commit/tag: {}", config.commit));

    let checkout_result = git_operations::checkout(&repo_path, &config.commit)?;

    if !checkout_result.is_success() {
        return Err(anyhow::anyhow!(
            "Failed to checkout {}: {}",
            config.commit,
            checkout_result.stderr
        ));
    }

    Ok(())
}
