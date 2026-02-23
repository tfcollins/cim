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

// Message utility functions for consistent output formatting
use std::sync::atomic::{AtomicBool, Ordering};

/// Global verbose flag
static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Standard success icon
pub const SUCCESS: &str = "✓";
/// Standard error icon  
pub const ERROR: &str = "✗";
/// Standard warning/info icon
pub const INFO: &str = "!";

/// Set verbose mode
pub fn set_verbose(enabled: bool) {
    VERBOSE.store(enabled, Ordering::SeqCst);
}

/// Check if verbose mode is enabled
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

/// Print a success message with standard formatting
pub fn success(msg: &str) {
    println!("{} {}", SUCCESS, msg);
}

/// Print an error message with standard formatting
pub fn error(msg: &str) {
    eprintln!("{} {}", ERROR, msg);
}

/// Print an info/warning message with standard formatting
pub fn info(msg: &str) {
    println!("{} {}", INFO, msg);
}

/// Print a simple status message without icon
pub fn status(msg: &str) {
    println!("{}", msg);
}

/// Print verbose message (only shown when verbose mode is enabled)
pub fn verbose(msg: &str) {
    if is_verbose() {
        println!("  {}", msg);
    }
}

/// Print detailed progress with repository name and action
pub fn progress(repo: &str, action: &str) {
    if is_verbose() {
        println!("  {} - {}", repo, action);
    }
}

/// Print workspace status line
pub fn workspace(path: &std::path::Path) {
    status(&format!("Workspace: {}", path.display()));
}
