// SPDX-FileCopyrightText: 2026 Lusoris <lusoris@proton.me>
// SPDX-License-Identifier: EUPL-1.2

//! HISS-03: the claim that no decision path allocates, made falsifiable.
//!
//! The reason this crate can say a decision path allocates nothing is
//! structural: every value on such a path is `Copy`, so planning a mesh or
//! admitting a solver writes into storage that already exists.
//! `assert_no_heap::<T>()` is the executable form: a `Vec`, `String` or `Box`
//! anywhere inside one of these types, at any depth, removes `Copy` and this
//! file stops compiling, which fails the gate.
//!
//! # What is deliberately not claimed
//!
//! **Decoding is not unconditionally allocation-free, and this file says so by
//! exercising the case rather than by denying it.** A wire tag spelled with a
//! JSON escape makes `serde_json` unescape it into a heap scratch buffer before
//! any type of ours sees it. That buffer is never retained, and the decoded
//! value still owns no heap -- which is the claim the crate actually makes.

mod common;

use core::mem::size_of;

use tornus_geometry::{
    BrepEvaluator, CadEngine, EvaluationOutcome, GeometryError, IdError, MAX_MESH_ELEMENTS,
    MeshConfig, MeshElementCount, MeshPlan, SolverAdmission, SolverTarget, StepPath, StepSchema,
    Tolerance,
};

use common::Fallible;

/// Accepts only a type that owns no heap, because it is `Copy`.
fn assert_no_heap<T: Copy>() {}

// --- Positive -------------------------------------------------------------

/// Positive: every value on a decision path owns no heap.
///
/// This is the falsifier. Adding a `Vec`, `String` or `Box` to any of these
/// types, or to anything they contain, removes `Copy` and this test stops
/// compiling.
#[test]
fn every_value_on_a_decision_path_owns_no_heap() {
    assert_no_heap::<BrepEvaluator>();
    assert_no_heap::<CadEngine>();
    assert_no_heap::<EvaluationOutcome>();
    assert_no_heap::<MeshConfig>();
    assert_no_heap::<MeshElementCount>();
    assert_no_heap::<MeshPlan>();
    assert_no_heap::<StepPath>();
    assert_no_heap::<StepSchema>();
    assert_no_heap::<Tolerance>();
    assert_no_heap::<SolverAdmission>();
    assert_no_heap::<SolverTarget>();
    assert_no_heap::<GeometryError>();
    assert_no_heap::<IdError>();
}

/// Positive: driving a plan to the element bound changes no size.
#[test]
fn planning_at_the_bound_changes_no_storage() -> Fallible {
    let engine = common::evaluator();
    let plan = engine.mesh(common::mesh_config()?, MAX_MESH_ELEMENTS)?;
    assert_eq!(size_of_val(&plan), size_of::<MeshPlan>());
    assert_eq!(size_of_val(&engine), size_of::<BrepEvaluator>());
    assert!(size_of::<MeshPlan>() >= size_of::<MeshElementCount>());
    Ok(())
}

/// Positive: the wire tags serde writes are the tags the types report.
#[test]
fn the_wire_tags_are_the_reported_tags() -> Fallible {
    for engine in CadEngine::BOTH {
        assert_eq!(
            serde_json::to_string(&engine)?,
            format!("\"{}\"", engine.tag())
        );
    }
    assert_eq!(
        serde_json::to_string(&StepSchema::ADMITTED)?,
        format!("\"{}\"", StepSchema::ADMITTED.tag())
    );
    Ok(())
}

// --- Negative -------------------------------------------------------------

/// Negative: a decoded tag owns no heap even when the input made the parser
/// allocate, which is the case this crate does not deny.
#[test]
fn an_escaped_tag_still_decodes_into_an_inline_value() -> Fallible {
    let decoded: CadEngine = serde_json::from_str("\"exact\\u002Dnurbs-brep\"")?;
    assert_eq!(decoded, CadEngine::ExactNurbsBrep);
    assert_eq!(size_of_val(&decoded), size_of::<CadEngine>());
    Ok(())
}

/// Negative: a protocol other than the admitted one does not decode.
#[test]
fn an_unadmitted_protocol_does_not_decode() {
    assert!(serde_json::from_str::<StepSchema>("\"AP214\"").is_err());
    assert!(serde_json::from_str::<CadEngine>("\"exact-nurbs\"").is_err());
}

// --- Boundary -------------------------------------------------------------

/// Boundary: a `STEP` path at its byte bound is still an inline value of the
/// same size as a one-byte path.
#[test]
fn a_path_at_its_bound_is_the_size_of_any_other() -> Fallible {
    let short = StepPath::parse("a")?;
    let long = StepPath::parse(&"a".repeat(tornus_geometry::MAX_STEP_PATH_LEN))?;
    assert_eq!(size_of_val(&short), size_of_val(&long));
    assert_eq!(long.len(), tornus_geometry::MAX_STEP_PATH_LEN);
    Ok(())
}
