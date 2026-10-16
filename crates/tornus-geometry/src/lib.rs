// SPDX-FileCopyrightText: 2026 Lusoris <lusoris@proton.me>
// SPDX-License-Identifier: EUPL-1.2

//! `tornus-geometry`: the bounded geometry and meshing limits.
//!
//! *Tornus* is Latin for a turner's lathe (TOR-nus); this crate is the part
//! of it that holds the limits before any tool turns.
//!
//! The code moved from the Aegis OS `aegis-hephaestus` crate (placement
//! CUT-1632-01). What moved is the part that can be checked without a `CAD`
//! kernel, a mesher or a solver -- the bounds and the admissions -- with each
//! refusal a typed value. What a geometry request means on a consumer's wire
//! (the verification request it answers, the viewport descriptor it hands on,
//! the requirement register) stays with that consumer.
//!
//! # The three things a reviewer should look at
//!
//! * [`BrepEvaluator`] holds both bounds against caller-supplied input:
//!   [`MAX_GEOMETRY_ITERATIONS`] stops a walk that would run past it, and
//!   [`MAX_MESH_ELEMENTS`] accepts the 500,000th element and refuses the
//!   500,001st. A request that names no `STEP` source is refused before either.
//! * [`MeshElementCount`] is the meshing bound as a value, so a consumer that
//!   puts an element count on a wire carries the same bound as the planner.
//! * [`SolverAdmission`] names three solvers and starts none. There is
//!   deliberately no dispatch method anywhere in this crate.
//!
//! # Fixed point, not floating point
//!
//! Every length here is an integer count of micrometres. A boundary test on a
//! binary float is a test on rounding.
//!
//! # Design invariants (see `AGENTS.md`)
//!
//! * no recursion: every function here is a flat check, a bounded sweep or a
//!   field access, and no function in the crate calls itself directly or
//!   through another;
//! * every loop carries a scalar upper bound: [`MAX_GEOMETRY_ITERATIONS`] and
//!   [`MAX_STEP_PATH_LEN`];
//! * every value on a decision path is `Copy`, so no such path allocates;
//!   `tests/allocation_bounds.rs` is the falsifier;
//! * no `unwrap`, `expect`, `panic!`, slice indexing or unchecked arithmetic,
//!   and no `unsafe` (forbidden at the workspace root).
//!
//! # What this crate does not do
//!
//! It loads nothing and runs nothing. There is no `CAD` kernel, no `STEP`
//! parser, no mesher, no `Z3` or other solver, no process, no cgroup and no
//! filesystem access anywhere in it -- a `STEP` source is a validated name and
//! is never opened. `tests/stubbed_effects.rs` sweeps the crate's own sources
//! for a recorded list of identifiers that would be needed to do any of it and
//! fails if one appears outside a comment -- a regression gate over an
//! enumeration, not a proof over every such identifier.
//!
//! Nothing here returns a verdict: the evaluation outcome has no
//! `is_watertight`, because no kernel is linked that could say so.
//!
//! # Example
//!
//! ```
//! use tornus_geometry::{
//!     BrepEvaluator, CadEngine, EvaluationOutcome, MAX_MESH_ELEMENTS, MeshConfig, StepPath,
//!     Tolerance,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let evaluator = BrepEvaluator::new(CadEngine::ExactNurbsBrep, Tolerance::SCAFFOLD);
//! let source = StepPath::parse("/var/lib/geometry/bracket.step")?;
//!
//! let walked = evaluator.evaluate(Some(source), 120)?;
//! assert_eq!(walked, EvaluationOutcome::Completed { iterations: 120 });
//! assert!(evaluator.evaluate(None, 120).is_err());
//!
//! let config = MeshConfig::new(100, 2_000, true)?;
//! let plan = evaluator.mesh(config, 120_000)?;
//! assert_eq!(plan.elements.get(), 120_000);
//! assert!(evaluator.mesh(config, MAX_MESH_ELEMENTS).is_ok());
//! assert!(evaluator.mesh(config, MAX_MESH_ELEMENTS + 1).is_err());
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod geometry;
pub mod id;
pub mod solver;

pub use crate::error::GeometryError;
pub use crate::geometry::{
    BrepEvaluator, CadEngine, EvaluationOutcome, MAX_GEOMETRY_ITERATIONS, MAX_MESH_ELEMENTS,
    MAX_TOLERANCE_MICROMETRES, MIN_TOLERANCE_MICROMETRES, MeshConfig, MeshElementCount, MeshPlan,
    StepSchema, Tolerance,
};
pub use crate::id::{IdError, MAX_STEP_PATH_LEN, StepPath};
pub use crate::solver::{SOLVER_SLICE, SolverAdmission, SolverTarget};
