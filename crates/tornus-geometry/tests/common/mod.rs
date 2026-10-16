// SPDX-FileCopyrightText: 2026 Lusoris <lusoris@proton.me>
// SPDX-License-Identifier: EUPL-1.2

//! Fixtures shared by the integration tests.
//!
//! Everything here reaches `tornus-geometry` through its public API only, and
//! nothing here uses `unwrap` or `expect`: a test that cannot build its own
//! input returns the failure instead of panicking through a lint the workspace
//! denies.

#![allow(dead_code)]

use tornus_geometry::{BrepEvaluator, CadEngine, MeshConfig, StepPath, Tolerance};

/// What every test in this suite returns.
pub type Fallible = Result<(), Box<dyn std::error::Error>>;

/// The `STEP` source the fixtures name and never open.
pub const STEP_SOURCE: &str = "/var/lib/geometry/bracket.step";

/// The element count the scaffold hard-codes as its meshing result.
pub const SCAFFOLD_ELEMENTS: usize = 120_000;

/// Returns the fixture `STEP` source path.
///
/// # Errors
///
/// Propagates [`StepPath::parse`].
pub fn step_source() -> Result<StepPath, Box<dyn std::error::Error>> {
    Ok(StepPath::parse(STEP_SOURCE)?)
}

/// Returns an evaluator for the exact boundary-representation engine.
#[must_use]
pub fn evaluator() -> BrepEvaluator {
    BrepEvaluator::new(CadEngine::ExactNurbsBrep, Tolerance::SCAFFOLD)
}

/// Returns the mesh configuration the fixtures use, in micrometres.
///
/// # Errors
///
/// Propagates [`MeshConfig::new`].
pub fn mesh_config() -> Result<MeshConfig, Box<dyn std::error::Error>> {
    Ok(MeshConfig::new(100, 2_000, true)?)
}

/// Scalar bound on directories visited and entries read per directory.
pub const WALK_BOUND: usize = 64;

/// Scalar bound on the lines read from any one source file.
pub const LINE_BOUND: usize = 4096;

/// Returns the `.rs` files under `root`, as (file name, body) pairs.
///
/// The walk is iterative and doubly bounded: at most [`WALK_BOUND`]
/// directories and at most [`WALK_BOUND`] entries per directory.
///
/// # Errors
///
/// Returns the first read failure, so a sweep cannot pass by reading nothing.
pub fn rust_sources(
    root: &std::path::Path,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut stack: Vec<std::path::PathBuf> = vec![root.to_path_buf()];
    for _ in 0..WALK_BOUND {
        let Some(directory) = stack.pop() else { break };
        for entry in std::fs::read_dir(&directory)?.take(WALK_BOUND) {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let name = path
                .file_name()
                .map(|raw| raw.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push((name, std::fs::read_to_string(&path)?));
        }
    }
    if out.is_empty() {
        return Err(format!("no Rust source was read under {}", root.display()).into());
    }
    Ok(out)
}

/// Returns the `"<file>:<line>: <identifier>"` hits `text` carries.
fn hits(name: &str, number: usize, text: &str, watched: &[&str]) -> Vec<String> {
    watched
        .iter()
        .filter(|needle| text.contains(**needle))
        .map(|needle| format!("{name}:{}: {needle}", number.saturating_add(1)))
        .collect()
}

/// Returns the hits one body of text carries, skipping comment lines.
///
/// Comment lines are skipped whole: the crate documentation names the very
/// identifiers a sweep watches for, precisely in order to say it does not use
/// them, so a sweep that read comments would fail on its own documentation.
///
/// This is public so a watched list can be exercised against planted text.
pub fn scan_text(name: &str, body: &str, watched: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (number, line) in body.lines().take(LINE_BOUND).enumerate() {
        let text = line.trim();
        if text.starts_with("//") {
            continue;
        }
        found.extend(hits(name, number, text, watched));
    }
    found
}

/// Returns every watched identifier the sources under `root` carry.
///
/// # Errors
///
/// Propagates [`rust_sources`], so a sweep cannot pass by reading nothing.
pub fn scan_sources(
    root: &std::path::Path,
    watched: &[&str],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut found: Vec<String> = Vec::new();
    for (name, body) in rust_sources(root)? {
        found.extend(scan_text(&name, &body, watched));
    }
    Ok(found)
}

/// Returns the crate's own `src/` directory.
#[must_use]
pub fn source_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}
