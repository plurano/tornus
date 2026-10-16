<!--
SPDX-FileCopyrightText: 2026 Lusoris <lusoris@pm.me>
SPDX-License-Identifier: EUPL-1.2
-->

# Tornus

Tornus is a geometric modelling kernel for CAD, written in Rust: a correct
modelling core checked against a public conformance corpus. It starts from the
Aegis OS crate `aegis-hephaestus`, which today declares the bounded geometry
and meshing limits and the solver slice; that crate moves here and Aegis OS
depends on this library instead.

## The name

*Tornus* is Latin for a lathe, the turner's wheel (from Greek *tornos*); a lathe
shapes a solid by turning it. It is pronounced TOR-noos.

## Status

No code yet. The repository stays private until its first release.

## Licence

Licensed under the European Union Public Licence 1.2
([LICENSES/EUPL-1.2.txt](LICENSES/EUPL-1.2.txt)). The repository follows the
[REUSE](https://reuse.software) specification.

<!-- praetor:readme-governance:start -->
[![HISS Adopted][praetor-hiss-badge]][praetor-hiss-agents]

Praetor manages this repository's declared governance policy. This managed
block records adoption state; it is not a verification certificate.

**Verification**: `make verify-all` runs the repository's configured
verification cascade.

**HISS Audit**: `praetorctl audit` enforces policy, generated-surface
integrity, and the debt ratchet.

**Context Sync**: `praetorctl compile-context --verify` verifies every
generated agent context against `AGENTS.md`.

**Debt Baseline**: `.standards-baseline.json` anchors the debt ratchet at
0 recorded infractions; audit forbids growth.

[praetor-hiss-badge]: https://img.shields.io/badge/Standards-HISS%20Adopted-blue
[praetor-hiss-agents]: https://github.com/plurano/tornus/blob/HEAD/AGENTS.md
<!-- praetor:readme-governance:end -->
