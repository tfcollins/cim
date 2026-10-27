// Copyright (c) 2026 Analog Devices, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use dsdk_cli::config::{
    self, CopyFileConfig, MakefileInclude, OsDependencies, PythonDependencies, SdkConfig,
    SdkConfigCore,
};
use dsdk_cli::messages;
use dsdk_cli::workspace::{
    self, get_all_sources, load_config_with_extends, WorkspaceMarker, OVERLAYS_DIR,
    OS_DEPS_FILE, PYTHON_DEPS_FILE, SDK_CONFIG_FILE, WORKSPACE_MARKER_FILE,
};

use crate::init_cmd::{self, extract_target_description};

#[derive(Debug)]
pub struct InspectConfig {
    pub target: Option<String>,
    pub source: Option<String>,
    pub version: Option<String>,
    pub format: String,
    pub verbose: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableInfo {
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectGitInfo {
    pub name: String,
    pub url: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_deps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectToolchainInfo {
    pub name: String,
    pub url: String,
    pub destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectInstallInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on_gits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentinel: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TargetInspectResult {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extends_chain: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub variables: BTreeMap<String, VariableInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub gits: Vec<InspectGitInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub toolchains: Vec<InspectToolchainInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub install: Vec<InspectInstallInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub copy_files: Vec<CopyFileConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub makefile_include: Option<MakefileInclude>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub phases: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub phase_commands: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_dependencies: Option<OsDependencies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_dependencies: Option<PythonDependencies>,
}

/// Extract variable doc comments from YAML text preceding each variable in `variables:`
pub fn extract_variable_comments(sdk_path: &Path) -> HashMap<String, String> {
    let mut comments = HashMap::new();
    let content = match fs::read_to_string(sdk_path) {
        Ok(c) => c,
        Err(_) => return comments,
    };

    let mut in_variables = false;
    let mut current_comment: Vec<String> = Vec::new();

    for line in content.lines() {
        let stripped = line.trim();
        if stripped.is_empty() {
            current_comment.clear();
            continue;
        }

        if stripped.starts_with("variables:") {
            in_variables = true;
            current_comment.clear();
            continue;
        }

        if in_variables {
            // If line is unindented and not a comment, we've exited variables:
            if !line.starts_with(' ') && !line.starts_with('\t') && !stripped.starts_with('#') {
                in_variables = false;
                current_comment.clear();
                continue;
            }

            if stripped.starts_with('#') {
                let comment_text = stripped.trim_start_matches('#').trim();
                if !comment_text.is_empty() {
                    current_comment.push(comment_text.to_string());
                }
                continue;
            }

            // Look for variable declaration: VAR_NAME: value
            if let Some(colon_pos) = line.find(':') {
                let prefix = line[..colon_pos].trim();
                if !prefix.is_empty()
                    && prefix
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    let var_name = prefix.to_string();
                    if !current_comment.is_empty() {
                        comments.insert(var_name, current_comment.join(" "));
                        current_comment.clear();
                    }
                } else {
                    current_comment.clear();
                }
            } else {
                current_comment.clear();
            }
        }
    }

    comments
}

/// Main entry point for the `cim inspect` command
pub fn handle_inspect_command(cfg: InspectConfig) {
    messages::set_verbose(cfg.verbose);
    let json = cfg.format.eq_ignore_ascii_case("json");

    if let Some(ref target_name) = cfg.target {
        inspect_target(target_name, &cfg, json);
    } else if let Some(workspace_root) = workspace::find_workspace_root() {
        inspect_workspace(&workspace_root, json);
    } else {
        messages::error("Target name is required when not running inside an initialized workspace.");
        messages::info("Usage: cim inspect <TARGET> [--source DIR|URL]");
        messages::status("Use 'cim list-targets' to see available targets.");
        std::process::exit(1);
    }
}

/// Inspect target from manifest source(s)
fn inspect_target(target_name: &str, cfg: &InspectConfig, json: bool) {
    let sources = match &cfg.source {
        Some(s) => vec![s.clone()],
        None => get_all_sources(),
    };

    let resolved = match init_cmd::resolve_target_from_sources(
        target_name,
        cfg.version.as_deref(),
        &sources,
        None,
    ) {
        Ok(r) => r,
        Err(e) => {
            messages::error(&format!("Failed to resolve target '{}': {}", target_name, e));
            std::process::exit(1);
        }
    };

    let target_dir = resolved
        .config_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let primary_sdk = match config::load_config(&resolved.config_path) {
        Ok(s) => s,
        Err(e) => {
            messages::error(&format!(
                "Failed to load config for target '{}': {}",
                target_name, e
            ));
            std::process::exit(1);
        }
    };

    let resolution = match init_cmd::resolve_extends_chain_from_source(
        &resolved.config_path,
        target_name,
        &sources,
    ) {
        Ok(res) => res,
        Err(e) => {
            messages::error(&format!(
                "Failed to resolve extends chain for target '{}': {}",
                target_name, e
            ));
            std::process::exit(1);
        }
    };

    let merged = resolution.merged;

    // Trace extends chain and collect variable comments across inheritance
    let mut extends_chain = Vec::new();
    let mut variable_comments = HashMap::new();

    // Files in resolution.files_to_copy are [primary_target_files..., ancestor_files...]
    // To have child comments override ancestor comments, parse ancestors first then primary
    for file_pair in resolution.files_to_copy.iter().rev() {
        if file_pair.dest_name.ends_with(SDK_CONFIG_FILE) {
            let comments = extract_variable_comments(&file_pair.source_path);
            variable_comments.extend(comments);
        }
    }

    // Trace extends chain target names
    if let Some(ref ext) = primary_sdk.extends {
        extends_chain.push(ext.target.clone());
        let mut current_target = ext.target.clone();
        for _ in 0..20 {
            let ancestor_file_name = format!("{}-{}", current_target, SDK_CONFIG_FILE);
            if let Some(pair) = resolution
                .files_to_copy
                .iter()
                .find(|p| p.dest_name == ancestor_file_name)
            {
                if let Ok(anc_sdk) = config::load_config(&pair.source_path) {
                    if let Some(next_ext) = anc_sdk.extends {
                        extends_chain.push(next_ext.target.clone());
                        current_target = next_ext.target;
                        continue;
                    }
                }
            }
            break;
        }
    }

    let description = extract_target_description(target_dir, Some(&primary_sdk))
        .or_else(|| merged.description.clone());

    let inspect_result = build_target_inspect_result(
        target_name.to_string(),
        cfg.version.clone(),
        Some(resolved.source_path),
        None,
        description,
        primary_sdk.extends.as_ref().map(|e| e.target.clone()),
        extends_chain,
        &merged,
        &variable_comments,
        target_dir,
        &resolution.files_to_copy,
    );

    output_result(&inspect_result, json);
}

/// Inspect target from an existing workspace
fn inspect_workspace(workspace_root: &Path, json: bool) {
    let marker_path = workspace_root.join(WORKSPACE_MARKER_FILE);
    let marker: Option<WorkspaceMarker> = if marker_path.exists() {
        fs::read_to_string(&marker_path)
            .ok()
            .and_then(|c| noyalib::from_str(&c).ok())
    } else {
        None
    };

    let target_name = marker
        .as_ref()
        .map(|m| m.target.clone())
        .unwrap_or_else(|| "workspace".to_string());
    let target_version = marker.as_ref().map(|m| m.target_version.clone());
    let source = marker.as_ref().and_then(|m| m.config_source_dir.clone());

    let sdk_path = workspace_root.join(SDK_CONFIG_FILE);
    if !sdk_path.exists() {
        messages::error(&format!(
            "Missing {} in workspace: {}",
            SDK_CONFIG_FILE,
            workspace_root.display()
        ));
        std::process::exit(1);
    }

    let primary_sdk = match config::load_config(&sdk_path) {
        Ok(s) => s,
        Err(e) => {
            messages::error(&format!("Failed to load workspace sdk.yml: {}", e));
            std::process::exit(1);
        }
    };

    let merged = match load_config_with_extends(&sdk_path) {
        Ok(m) => m,
        Err(e) => {
            messages::error(&format!("Failed to resolve workspace configuration: {}", e));
            std::process::exit(1);
        }
    };

    let mut variable_comments = HashMap::new();
    let mut extends_chain = Vec::new();

    // Check .cim/target-overlays/ for ancestor configs
    let overlays_dir = workspace_root.join(OVERLAYS_DIR);
    if overlays_dir.exists() {
        if let Ok(entries) = fs::read_dir(&overlays_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_string_lossy().ends_with(SDK_CONFIG_FILE) {
                    variable_comments.extend(extract_variable_comments(&path));
                }
            }
        }
    }
    variable_comments.extend(extract_variable_comments(&sdk_path));

    if let Some(ref ext) = primary_sdk.extends {
        extends_chain.push(ext.target.clone());
        let mut current_target = ext.target.clone();
        for _ in 0..20 {
            let anc_file = overlays_dir.join(format!("{}-{}", current_target, SDK_CONFIG_FILE));
            if anc_file.exists() {
                if let Ok(anc_sdk) = config::load_config(&anc_file) {
                    if let Some(next_ext) = anc_sdk.extends {
                        extends_chain.push(next_ext.target.clone());
                        current_target = next_ext.target;
                        continue;
                    }
                }
            }
            break;
        }
    }

    let description = extract_target_description(workspace_root, Some(&primary_sdk))
        .or_else(|| merged.description.clone());

    let files_to_copy = Vec::new();
    let inspect_result = build_target_inspect_result(
        target_name,
        target_version,
        source,
        Some(workspace_root.display().to_string()),
        description,
        primary_sdk.extends.as_ref().map(|e| e.target.clone()),
        extends_chain,
        &merged,
        &variable_comments,
        workspace_root,
        &files_to_copy,
    );

    output_result(&inspect_result, json);
}

fn build_target_inspect_result(
    target: String,
    version: Option<String>,
    source: Option<String>,
    workspace_path: Option<String>,
    description: Option<String>,
    extends: Option<String>,
    extends_chain: Vec<String>,
    merged: &SdkConfig,
    variable_comments: &HashMap<String, String>,
    base_dir: &Path,
    files_to_copy: &[init_cmd::TargetFilePair],
) -> TargetInspectResult {
    // Variables
    let mut variables = BTreeMap::new();
    if let Some(vars) = &merged.variables {
        for (k, v) in vars {
            let doc = variable_comments.get(k).cloned();
            variables.insert(
                k.clone(),
                VariableInfo {
                    default: v.clone(),
                    description: doc,
                },
            );
        }
    }

    // Gits
    let gits = merged
        .gits
        .iter()
        .map(|g| InspectGitInfo {
            name: g.name.clone(),
            url: g.url.clone(),
            commit: g.commit.clone(),
            build_depends_on: g.build_depends_on.clone(),
            git_depends_on: g.git_depends_on.clone(),
            build: g.build.clone(),
            python_deps: g.python_deps.clone(),
            group: g.group.clone(),
        })
        .collect();

    // Toolchains
    let toolchains = merged
        .toolchains
        .as_ref()
        .map(|tcs| {
            tcs.iter()
                .map(|tc| InspectToolchainInfo {
                    name: tc.get_name(),
                    url: tc.url.clone(),
                    destination: tc.destination.clone(),
                    os: tc.os.clone(),
                    arch: tc.arch.clone(),
                    sha256: tc.sha256.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    // Install
    let install = merged
        .install
        .as_ref()
        .map(|insts| {
            insts
                .iter()
                .map(|i| InspectInstallInfo {
                    name: i.name.clone(),
                    depends_on: i.depends_on.clone(),
                    depends_on_gits: i.depends_on_gits.clone(),
                    commands: i.commands.clone(),
                    sentinel: i.sentinel,
                })
                .collect()
        })
        .unwrap_or_default();

    // Copy files
    let copy_files = merged.copy_files.clone().unwrap_or_default();

    // Makefile include
    let makefile_include = merged.makefile_include.clone();

    // Phases & Phase Commands
    let phases = merged.phases();
    let mut phase_commands = BTreeMap::new();
    for phase in &phases {
        if let Some(target) = merged.phase_target(phase) {
            let cmds = target.commands().to_vec();
            if !cmds.is_empty() {
                phase_commands.insert(phase.clone(), cmds);
            }
        }
    }

    // OS dependencies
    let os_dependencies = files_to_copy
        .iter()
        .find(|p| p.dest_name == OS_DEPS_FILE)
        .map(|p| p.source_path.clone())
        .or_else(|| {
            let direct = base_dir.join(OS_DEPS_FILE);
            if direct.exists() {
                Some(direct)
            } else {
                None
            }
        })
        .and_then(|p| config::load_os_dependencies(&p).ok());

    // Python dependencies
    let python_dependencies = files_to_copy
        .iter()
        .find(|p| p.dest_name == PYTHON_DEPS_FILE)
        .map(|p| p.source_path.clone())
        .or_else(|| {
            let direct = base_dir.join(PYTHON_DEPS_FILE);
            if direct.exists() {
                Some(direct)
            } else {
                None
            }
        })
        .and_then(|p| config::load_python_dependencies(&p).ok());

    TargetInspectResult {
        target,
        version,
        source,
        workspace_path,
        description,
        extends,
        extends_chain,
        variables,
        gits,
        toolchains,
        install,
        copy_files,
        makefile_include,
        phases,
        phase_commands,
        os_dependencies,
        python_dependencies,
    }
}

fn output_result(res: &TargetInspectResult, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(res).expect("Failed to serialize inspect result to JSON")
        );
    } else {
        print_text_inspect(res);
    }
}

fn print_text_inspect(res: &TargetInspectResult) {
    println!("Target: {}", res.target);
    if let Some(ref desc) = res.description {
        println!("Description: {}", desc);
    }
    if let Some(ref src) = res.source {
        println!("Source: {}", src);
    }
    if let Some(ref ver) = res.version {
        println!("Version: {}", ver);
    }
    if let Some(ref ws) = res.workspace_path {
        println!("Workspace: {}", ws);
    }
    if let Some(ref ext) = res.extends {
        if res.extends_chain.len() > 1 {
            println!("Extends: {} (chain: {})", ext, res.extends_chain.join(" -> "));
        } else {
            println!("Extends: {}", ext);
        }
    }

    if !res.variables.is_empty() {
        println!("\nVariables:");
        for (name, info) in &res.variables {
            println!("  {} = \"{}\"", name, info.default);
            if let Some(ref desc) = info.description {
                println!("    # {}", desc);
            }
        }
    }

    if !res.gits.is_empty() {
        println!("\nRepositories (gits):");
        for git in &res.gits {
            println!("  - {} ({} @ {})", git.name, git.url, git.commit);
            if let Some(ref build) = git.build {
                if !build.is_empty() {
                    println!("    build: {}", build.join(" && "));
                }
            }
        }
    }

    if !res.toolchains.is_empty() {
        println!("\nToolchains:");
        for tc in &res.toolchains {
            println!("  - {} -> {}", tc.name, tc.destination);
            println!("    url: {}", tc.url);
        }
    }

    if !res.install.is_empty() {
        println!("\nInstall Steps:");
        for inst in &res.install {
            let deps_str = inst
                .depends_on
                .as_ref()
                .map(|d| format!(" [depends: {}]", d.join(", ")))
                .unwrap_or_default();
            println!("  - {}{}", inst.name, deps_str);
        }
    }

    if !res.copy_files.is_empty() {
        println!("\nCopy Files:");
        for cf in &res.copy_files {
            println!("  - {} -> {}", cf.source, cf.dest);
        }
    }

    if let Some(ref mi) = res.makefile_include {
        let files = mi.files();
        if !files.is_empty() {
            println!("\nMakefile Includes:");
            for f in files {
                println!("  - {}", f);
            }
        }
    }

    if !res.phases.is_empty() {
        println!("\nPhases: {}", res.phases.join(", "));
    }

    if let Some(ref os_deps) = res.os_dependencies {
        println!("\nOS Dependencies:");
        for (distro, cfg) in &os_deps.os_configs {
            for (subdistro, dcfg) in &cfg.distros {
                let pkgs = dcfg.package_manager.resolved_packages();
                println!(
                    "  {} / {} ({}): {}",
                    distro,
                    subdistro,
                    dcfg.package_manager.command,
                    pkgs.join(", ")
                );
            }
        }
    }

    if let Some(ref py_deps) = res.python_dependencies {
        println!("\nPython Dependencies:");
        for (profile, pcfg) in &py_deps.profiles {
            let is_default = if profile == &py_deps.default {
                " (default)"
            } else {
                ""
            };
            println!("  {}{}: {}", profile, is_default, pcfg.packages.join(", "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_extract_variable_comments() {
        let dir = tempdir().unwrap();
        let sdk_path = dir.path().join("sdk.yml");
        let content = r#"
variables:
  # Base platform name
  PLATFORM: zcu102
  # Build target
  # Multi-line comment
  TARGET: linux
  EMPTY_COMMENT: value
"#;
        fs::write(&sdk_path, content).unwrap();

        let comments = extract_variable_comments(&sdk_path);
        assert_eq!(
            comments.get("PLATFORM").unwrap(),
            "Base platform name"
        );
        assert_eq!(
            comments.get("TARGET").unwrap(),
            "Build target Multi-line comment"
        );
        assert!(!comments.contains_key("EMPTY_COMMENT"));
    }

    #[test]
    fn test_inspect_result_json_serialization() {
        let mut vars = BTreeMap::new();
        vars.insert(
            "PLATFORM".to_string(),
            VariableInfo {
                default: "zcu102".to_string(),
                description: Some("Base platform".to_string()),
            },
        );

        let res = TargetInspectResult {
            target: "adi-linux".to_string(),
            version: Some("main".to_string()),
            source: Some("/path/to/manifests".to_string()),
            workspace_path: None,
            description: Some("Linux target".to_string()),
            extends: Some("linux".to_string()),
            extends_chain: vec!["linux".to_string()],
            variables: vars,
            gits: vec![InspectGitInfo {
                name: "linux".to_string(),
                url: "https://example.com/linux.git".to_string(),
                commit: "master".to_string(),
                build_depends_on: None,
                git_depends_on: None,
                build: Some(vec!["make".to_string()]),
                python_deps: None,
                group: None,
            }],
            toolchains: Vec::new(),
            install: Vec::new(),
            copy_files: Vec::new(),
            makefile_include: None,
            phases: vec!["build".to_string()],
            phase_commands: BTreeMap::new(),
            os_dependencies: None,
            python_dependencies: None,
        };

        let json_str = serde_json::to_string_pretty(&res).unwrap();
        assert!(json_str.contains("\"target\": \"adi-linux\""));
        assert!(json_str.contains("\"PLATFORM\": {"));
        assert!(json_str.contains("\"default\": \"zcu102\""));
        assert!(json_str.contains("\"description\": \"Base platform\""));
        assert!(json_str.contains("\"extends\": \"linux\""));
    }

    #[test]
    fn test_extract_target_description_precedence() {
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join("my-target");
        fs::create_dir_all(&target_dir).unwrap();

        // 1. With explicit description in sdk.yml
        let sdk_with_desc = r#"
description: Explicit target description
variables:
  FOO: bar
"#;
        fs::write(target_dir.join(SDK_CONFIG_FILE), sdk_with_desc).unwrap();
        fs::write(
            target_dir.join("README.md"),
            "# Title\n\nREADME description text\n",
        )
        .unwrap();

        let sdk = config::load_config(&target_dir.join(SDK_CONFIG_FILE)).ok();
        let desc = extract_target_description(&target_dir, sdk.as_ref());
        assert_eq!(desc.unwrap(), "Explicit target description");

        // 2. Without explicit description, falls back to README.md
        let sdk_no_desc = r#"
# Comment header
variables:
  FOO: bar
"#;
        fs::write(target_dir.join(SDK_CONFIG_FILE), sdk_no_desc).unwrap();
        let sdk = config::load_config(&target_dir.join(SDK_CONFIG_FILE)).ok();
        let desc = extract_target_description(&target_dir, sdk.as_ref());
        assert_eq!(desc.unwrap(), "README description text");

        // 3. Without README.md, falls back to top comments in sdk.yml
        fs::remove_file(target_dir.join("README.md")).unwrap();
        let sdk = config::load_config(&target_dir.join(SDK_CONFIG_FILE)).ok();
        let desc = extract_target_description(&target_dir, sdk.as_ref());
        assert_eq!(desc.unwrap(), "Comment header");
    }

    #[test]
    fn test_list_available_target_summaries() {
        let dir = tempdir().unwrap();
        let targets_dir = dir.path().join("targets");
        let base_dir = targets_dir.join("base");
        let derived_dir = targets_dir.join("derived");
        fs::create_dir_all(&base_dir).unwrap();
        fs::create_dir_all(&derived_dir).unwrap();

        fs::write(
            base_dir.join(SDK_CONFIG_FILE),
            "description: Base target description\n",
        )
        .unwrap();
        fs::write(
            derived_dir.join(SDK_CONFIG_FILE),
            "extends: base\ndescription: Derived target description\n",
        )
        .unwrap();

        let summaries = init_cmd::list_available_target_summaries(dir.path()).unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].name, "base");
        assert_eq!(
            summaries[0].description.as_deref(),
            Some("Base target description")
        );
        assert_eq!(summaries[0].extends, None);

        assert_eq!(summaries[1].name, "derived");
        assert_eq!(
            summaries[1].description.as_deref(),
            Some("Derived target description")
        );
        assert_eq!(summaries[1].extends.as_deref(), Some("base"));
    }
}
