// SPDX-FileCopyrightText: 2026 Lusoris <lusoris@proton.me>
// SPDX-License-Identifier: EUPL-1.2

//! Executable checks on the workspace manifests.
//!
//! Three of this crate's claims are about files rather than about code:
//! `tornus-geometry` is a workspace member with a lock entry, the member list
//! is written out rather than globbed, and **the crate takes no `CAD` kernel,
//! no mesher, no solver binding, no process or clock dependency and no
//! dependency on the operating system it moved from**. Each is asserted here rather than
//! left as prose.

use std::path::{Path, PathBuf};

/// Returns the workspace root, two directories above this crate's manifest.
fn workspace_root() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    Some(manifest.parent()?.parent()?.to_path_buf())
}

/// Reads a workspace file, returning an empty string when it is missing.
fn read(name: &str) -> String {
    workspace_root()
        .and_then(|root| std::fs::read_to_string(root.join(name)).ok())
        .unwrap_or_default()
}

/// The member list is checked against this rather than against a literal
/// count. A count has to be edited every time a milestone activates a crate,
/// and hard-coding it in several crates at once makes all of them fail when
/// one member is added. What actually needs holding is
/// the invariant behind the count: every crate directory that has a manifest
/// is a workspace member, and nothing else is.
fn crate_directories_with_a_manifest() -> Vec<String> {
    let Some(root) = workspace_root() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(root.join("crates")) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .flatten()
        .take(64)
        .filter(|entry| entry.path().join("Cargo.toml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort_unstable();
    found
}

// --- Positive -------------------------------------------------------------

/// Positive: the workspace names this crate as a member.
#[test]
fn the_workspace_names_this_crate() {
    let manifest = read("Cargo.toml");
    assert!(!manifest.is_empty(), "the workspace manifest must exist");
    assert!(manifest.contains("\"crates/tornus-geometry\""));
}

/// Positive: the lock file carries an entry for this crate.
#[test]
fn the_lock_file_carries_an_entry_for_this_crate() {
    let lock = read("Cargo.lock");
    assert!(!lock.is_empty(), "Cargo.lock must be committed");
    assert!(lock.contains("name = \"tornus-geometry\""));
}

/// Positive: the crate declares the workspace's own three dependencies and
/// takes each from the workspace table.
#[test]
fn the_crate_declares_the_workspace_dependencies() {
    let manifest = read("crates/tornus-geometry/Cargo.toml");
    assert!(!manifest.is_empty(), "the crate manifest must exist");
    for expected in ["serde", "serde_json", "thiserror"] {
        assert!(manifest.contains(&format!("{expected}.workspace = true")));
    }
}

// --- Negative -------------------------------------------------------------

/// Negative: the member list is written out, never globbed.
#[test]
fn the_member_list_is_written_out_and_not_globbed() {
    let manifest = read("Cargo.toml");
    for glob in ["crates/*", "crates/**", "\"*\""] {
        assert!(
            !manifest.contains(glob),
            "the member list must be written out, not globbed with {glob}"
        );
    }
}

/// The dependencies this crate must never declare.
const EXCLUDED: [&str; 19] = [
    "truck",
    "truck-modeling",
    "opencascade",
    "occt-sys",
    "manifold3d",
    "csgrs",
    "gmsh",
    "gmsh-sys",
    "z3",
    "z3-sys",
    "bindgen",
    "cc",
    "cgroups-rs",
    "nix",
    "tokio",
    "libc",
    "chrono",
    "time",
    "aegis-",
];

/// Returns the excluded names `manifest` declares as dependencies.
///
/// The check is on declaration lines rather than on the whole file, because
/// the package description names several of these words in order to say the
/// crate does not use them -- a substring sweep would fail on the sentence
/// that makes the claim.
fn declared_exclusions(manifest: &str) -> Vec<&'static str> {
    let declarations: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| line.contains(" = ") || line.ends_with(".workspace = true"))
        .collect();
    EXCLUDED
        .into_iter()
        .filter(|excluded| {
            declarations.iter().any(|line| {
                line.starts_with(&format!("{excluded} "))
                    || line.starts_with(&format!("{excluded}."))
                    || (excluded.ends_with('-') && line.starts_with(excluded))
            })
        })
        .collect()
}

/// Negative: no CAD kernel, mesher, solver binding, process or clock
/// dependency, no build script, and no path back into the operating system
/// this code moved from.
#[test]
fn the_crate_manifest_declares_no_deferred_dependency() {
    let manifest = read("crates/tornus-geometry/Cargo.toml");
    assert!(!manifest.is_empty(), "the crate manifest must exist");
    assert_eq!(declared_exclusions(&manifest), Vec::<&str>::new());
    assert!(
        !manifest.contains("aegis"),
        "the crate names nothing of the operating system it moved from"
    );
    assert!(
        !manifest.contains("[build-dependencies]"),
        "a build script is how a foreign kernel or solver would arrive"
    );
}

/// Negative: the declaration check finds a planted dependency, so the test
/// above passing is not the check reading nothing.
#[test]
fn the_declaration_check_finds_a_planted_dependency() {
    let planted = "[dependencies]\n\
        aegis-justitia = { path = \"../aegis-justitia\" }\n\
        z3.workspace = true\n\
        description = \"links no z3\"\n";
    assert_eq!(declared_exclusions(planted), vec!["z3", "aegis-"]);
}

/// Negative: this crate ships a library, so there is no binary target.
#[test]
fn the_crate_declares_no_binary_target() {
    let manifest = read("crates/tornus-geometry/Cargo.toml");
    assert!(!manifest.contains("[[bin]]"));
    assert!(manifest.contains("[lib]"));
    assert!(manifest.contains("publish = false"));
}

// --- Boundary -------------------------------------------------------------

/// Boundary: the workspace lists exactly the crate directories that have a
/// manifest, no more and no fewer.
///
/// Not a hard-coded count: the number changes every time a crate is added. What must not change is that a directory with a manifest is a
/// member and nothing else is, so a new crate cannot be built by the workspace
/// without being named, and a named member cannot vanish from the tree.
#[test]
fn the_workspace_lists_every_crate_with_a_manifest() {
    let manifest = read("Cargo.toml");
    let members: Vec<String> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("\"crates/"))
        .map(|line| line.trim_matches(|c| c == '"' || c == ',').to_owned())
        .collect();
    let directories = crate_directories_with_a_manifest();
    assert!(!directories.is_empty(), "crates/ must hold manifests");
    assert_eq!(
        members.len(),
        directories.len(),
        "members {members:?} against directories {directories:?}"
    );
    for name in &directories {
        assert!(
            members
                .iter()
                .any(|member| member == &format!("crates/{name}")),
            "the member list must name {name}"
        );
    }
}

/// Boundary: the crate inherits the workspace lints rather than restating them.
#[test]
fn the_crate_inherits_the_workspace_lints() {
    let manifest = read("crates/tornus-geometry/Cargo.toml");
    assert!(manifest.contains("[lints]"));
    assert!(manifest.contains("workspace = true"));
    assert!(!manifest.contains("[lints.rust]"));
    assert!(!manifest.contains("[lints.clippy]"));
    let root = read("Cargo.toml");
    assert!(root.contains("unsafe_code = \"forbid\""));
}
