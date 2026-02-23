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

use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::process::Command;

#[derive(Default)]
pub struct ManifestFetcher;

impl ManifestFetcher {
    pub fn new() -> Self {
        Self
    }

    /// Fetch available targets using CLI command
    pub fn fetch_targets(&self, source: Option<&str>) -> Result<Vec<String>> {
        let mut cmd = Command::new("cim");
        cmd.arg("list-targets");
        
        if let Some(source_url) = source {
            cmd.arg("--source").arg(source_url);
        }

        let output = cmd.output()
            .map_err(|e| anyhow!("Failed to run cim list-targets: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "cim list-targets failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut targets = Vec::new();

        // Parse output - look for lines with "  - target_name"
        for line in stdout.lines() {
            let line = line.trim();
            if let Some(target_name) = line.strip_prefix("- ") {
                targets.push(target_name.trim().to_string());
            }
        }

        targets.sort();
        Ok(targets)
    }

    /// Fetch available versions for a specific target using CLI command
    pub fn fetch_target_versions(&self, source: Option<&str>, target: &str) -> Result<Vec<String>> {
        let mut cmd = Command::new("cim");
        cmd.arg("list-targets");
        cmd.arg("--target").arg(target);
        
        if let Some(source_url) = source {
            cmd.arg("--source").arg(source_url);
        }

        let output = cmd.output()
            .map_err(|e| anyhow!("Failed to run cim list-targets: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "cim list-targets failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut versions = Vec::new();

        // Parse output - look for lines with "  - version_name"
        for line in stdout.lines() {
            let line = line.trim();
            if let Some(version_name) = line.strip_prefix("- ") {
                versions.push(version_name.trim().to_string());
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Legacy method for HTTP-based manifest fetching (backward compatibility)
    pub fn fetch_manifest_list(&self, url: &str) -> Result<Vec<String>> {
        // Fallback to CLI-based approach
        self.fetch_targets(Some(url))
    }

    fn parse_manifest_list(&self, html_content: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html_content);

        // Try different selectors for common directory listing formats
        let selectors = [
            "a[href$='.yml']", // Links ending with .yml
            "a",               // All links (fallback)
        ];

        let mut yml_files = HashSet::new();

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        // Check if it's a YAML file
                        if href.ends_with(".yml") || href.ends_with(".yaml") {
                            let filename = href.trim_start_matches("./");

                            // Skip dependency files
                            if filename != "os-dependencies.yml"
                                && filename != "python-dependencies.yml"
                            {
                                yml_files.insert(filename.to_string());
                            }
                        }
                    }
                }
            }
        }

        // If no YAML files found with selectors, try parsing as plain text
        if yml_files.is_empty() {
            for line in html_content.lines() {
                let line = line.trim();
                if (line.ends_with(".yml") || line.ends_with(".yaml"))
                    && line != "os-dependencies.yml"
                    && line != "python-dependencies.yml"
                {
                    yml_files.insert(line.to_string());
                }
            }
        }

        if yml_files.is_empty() {
            return Err(anyhow!("No manifest files found at the specified URL"));
        }

        // Convert to display names (remove .yml extension) and sort
        let mut projects: Vec<String> = yml_files
            .into_iter()
            .map(|filename| {
                filename
                    .strip_suffix(".yml")
                    .or_else(|| filename.strip_suffix(".yaml"))
                    .unwrap_or(&filename)
                    .to_string()
            })
            .collect();

        projects.sort();
        Ok(projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest_list() {
        let fetcher = ManifestFetcher::new();

        let html = r#"
        <html>
        <body>
        <a href="foobar.yml">foobar.yml</a>
        <a href="superduper.yml">superduper.yml</a>
        <a href="optee-qemu-v8.yml">optee-qemu-v8.yml</a>
        <a href="os-dependencies.yml">os-dependencies.yml</a>
        <a href="python-dependencies.yml">python-dependencies.yml</a>
        </body>
        </html>
        "#;

        let result = fetcher.parse_manifest_list(html).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"foobar".to_string()));
        assert!(result.contains(&"superduper".to_string()));
        assert!(result.contains(&"optee-qemu-v8".to_string()));
        assert!(!result.contains(&"os-dependencies".to_string()));
        assert!(!result.contains(&"python-dependencies".to_string()));
    }

    #[test]
    fn test_parse_plain_text_listing() {
        let fetcher = ManifestFetcher::new();

        let text = r#"
        foobar.yml
        superduper.yml
        optee-qemu-v8.yml
        os-dependencies.yml
        python-dependencies.yml
        "#;

        let result = fetcher.parse_manifest_list(text).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"foobar".to_string()));
        assert!(result.contains(&"superduper".to_string()));
        assert!(result.contains(&"optee-qemu-v8".to_string()));
    }
}
