// SPDX-FileCopyrightText: 2026 Lusoris <lusoris@proton.me>
// SPDX-License-Identifier: EUPL-1.2

//! The wire form of the bounded values, at their edges.
//!
//! Positive: a source path, an element count and a tolerance round-trip
//! through JSON unchanged. Negative: a value its constructor refuses is
//! refused by the decoder too, with the decoder's own reason, and a value of
//! the wrong JSON kind does not decode. Boundary: the last admissible element
//! count, tolerance and path length decode, and the first inadmissible ones do
//! not.

mod common;

use tornus_geometry::{
    MAX_MESH_ELEMENTS, MAX_STEP_PATH_LEN, MAX_TOLERANCE_MICROMETRES, MIN_TOLERANCE_MICROMETRES,
    MeshElementCount, StepPath, Tolerance,
};

use common::{Fallible, SCAFFOLD_ELEMENTS, STEP_SOURCE};

/// Decodes `text` as `T` and returns the decoder's refusal, if any.
fn refusal<T: serde::de::DeserializeOwned>(text: &str) -> Option<String> {
    serde_json::from_str::<T>(text)
        .err()
        .map(|error| error.to_string())
}

/// Returns a JSON string holding a path of `len` permitted bytes.
fn path_of(len: usize) -> String {
    format!("\"/{}\"", "a".repeat(len.saturating_sub(1)))
}

// --- Positive -------------------------------------------------------------

/// Positive: a source path travels as its own text.
#[test]
fn a_source_path_round_trips_as_text() -> Fallible {
    let path = StepPath::parse(STEP_SOURCE)?;
    let text = serde_json::to_string(&path)?;
    assert_eq!(text, format!("\"{STEP_SOURCE}\""));
    assert_eq!(serde_json::from_str::<StepPath>(&text)?, path);
    Ok(())
}

/// Positive: element counts and tolerances travel as plain integers.
#[test]
fn bounded_numbers_round_trip_as_integers() -> Fallible {
    let elements = MeshElementCount::new(SCAFFOLD_ELEMENTS)?;
    assert_eq!(serde_json::to_string(&elements)?, "120000");
    assert_eq!(
        serde_json::from_str::<MeshElementCount>("120000")?,
        elements
    );

    let tolerance = Tolerance::new(25)?;
    assert_eq!(serde_json::to_string(&tolerance)?, "25");
    assert_eq!(serde_json::from_str::<Tolerance>("25")?, tolerance);
    Ok(())
}

// --- Negative -------------------------------------------------------------

/// Negative: a path the constructor refuses is refused by the decoder, which
/// names what it expected.
#[test]
fn a_refused_path_does_not_decode() {
    for text in ["\"\"", "\"bracket step\"", "\"../a;b\""] {
        let reason = refusal::<StepPath>(text);
        assert!(
            reason
                .as_deref()
                .is_some_and(|reason| reason.contains("a bounded STEP source path")),
            "{text} decoded or was refused for another reason: {reason:?}"
        );
    }
}

/// Negative: numbers of the wrong kind or sign are refused, not coerced.
#[test]
fn numbers_of_the_wrong_kind_do_not_decode() {
    for text in ["-1", "1.5", "\"25\"", "null"] {
        assert!(refusal::<Tolerance>(text).is_some(), "{text} decoded");
        assert!(
            refusal::<MeshElementCount>(text).is_some(),
            "{text} decoded"
        );
    }
    assert!(refusal::<StepPath>("42").is_some());
}

// --- Boundary -------------------------------------------------------------

/// Boundary: the element bound is closed at its edge.
#[test]
fn the_element_bound_is_closed_at_its_edge() -> Fallible {
    let last = MAX_MESH_ELEMENTS.to_string();
    assert_eq!(
        serde_json::from_str::<MeshElementCount>(&last)?.get(),
        MAX_MESH_ELEMENTS
    );
    let first_out = MAX_MESH_ELEMENTS.saturating_add(1).to_string();
    let reason = refusal::<MeshElementCount>(&first_out);
    assert!(
        reason
            .as_deref()
            .is_some_and(|reason| reason.contains("inside the declared bound")),
        "{first_out} decoded or was refused for another reason: {reason:?}"
    );
    Ok(())
}

/// Boundary: the tolerance range is closed at both edges.
#[test]
fn the_tolerance_range_is_closed_at_both_edges() -> Fallible {
    for edge in [MIN_TOLERANCE_MICROMETRES, MAX_TOLERANCE_MICROMETRES] {
        assert_eq!(
            serde_json::from_str::<Tolerance>(&edge.to_string())?.get(),
            edge
        );
    }
    let below = MIN_TOLERANCE_MICROMETRES.saturating_sub(1).to_string();
    let above = MAX_TOLERANCE_MICROMETRES.saturating_add(1).to_string();
    for text in [below, above, u64::MAX.to_string()] {
        let reason = refusal::<Tolerance>(&text);
        assert!(
            reason
                .as_deref()
                .is_some_and(|reason| reason.contains("inside the declared range")),
            "{text} decoded or was refused for another reason: {reason:?}"
        );
    }
    Ok(())
}

/// Boundary: a path of exactly the bound decodes and one byte more does not.
#[test]
fn the_path_bound_is_closed_at_its_edge() -> Fallible {
    let last = serde_json::from_str::<StepPath>(&path_of(MAX_STEP_PATH_LEN))?;
    assert_eq!(last.as_bytes().len(), MAX_STEP_PATH_LEN);
    let reason = refusal::<StepPath>(&path_of(MAX_STEP_PATH_LEN.saturating_add(1)));
    assert!(
        reason
            .as_deref()
            .is_some_and(|reason| reason.contains("a bounded STEP source path")),
        "a path past the bound decoded or was refused for another reason: {reason:?}"
    );
    Ok(())
}
