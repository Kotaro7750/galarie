<!-- Sync Impact Report
Version: 0.0.0 → 1.0.0
Modified Principles:
- PRINCIPLE_1_NAME -> Principle I - Independently Valuable Slices
- PRINCIPLE_2_NAME -> Principle II - Research-Led Planning
- PRINCIPLE_3_NAME -> Principle III - Contract-First Interfaces
- PRINCIPLE_4_NAME -> Principle IV - Test-Gated Implementation
- PRINCIPLE_5_NAME -> Principle V - Operational Transparency & Versioning
Added Sections:
- Delivery Constraints
- Delivery Workflow & Quality Gates
Removed Sections:
- None
Templates requiring updates (✅ updated / ⚠ pending):
- ✅ .specify/templates/plan-template.md
- ✅ .specify/templates/spec-template.md
- ✅ .specify/templates/tasks-template.md
Follow-up TODOs:
- None
-->
# Galarie Constitution

## Core Principles

### Principle I - Independently Valuable Slices
All feature specs, plans, and task lists MUST decompose work into prioritized,
independently deliverable user stories with their own acceptance tests,
observable outcomes, and rollback plans. Reject any feature request that fails
to articulate user-facing value or cannot be validated via a single CLI/API
journey. Rationale: the repository only ships work that can be demonstrated,
tested, and reverted without collateral impact.

### Principle II - Research-Led Planning
Delivery begins with documented research, risks, and constraints before any
task is scheduled. Implementation plans must cite research findings, codify
technical context (language, dependencies, performance budgets), and expose
open questions. Rationale: every build decision must be defensible, repeatable,
and grounded in recorded evidence that future contributors can audit.

### Principle III - Contract-First Interfaces
APIs, CLIs, and data models are specified, versioned, and prototyped (via
contracts/, quickstarts, and data-model docs) before code lands in src/. Each
contract must define schemas, error surfaces, and test harnesses so consumers
and producers evolve in lockstep. Rationale: explicit contracts prevent
integration drift and keep downstream automation trustworthy.

### Principle IV - Test-Gated Implementation
No code change merges without failing tests written first, covering contract,
integration, and critical unit paths tied to their user story. Tests execute as
part of the plan (Phase 0/1) and tasks (Phase 3+) so Red -> Green -> Refactor is
observable in review. Rationale: regressions become visible earlier than
release and reviewers can trace behavior to enforceable evidence.

### Principle V - Operational Transparency & Versioning
Every change carries structured logging, metrics, and documentation updates
that show how the system performs in production-like environments. All artifacts
(plans, specs, contracts, code) must reference semantic versions and changelog
entries so stakeholders understand blast radius. Rationale: observable systems
enable faster debugging, safer rollbacks, and accountable governance.

## Delivery Constraints

1. Features flow through `/specs/<feature>/` with mandatory `research.md`,
   `plan.md`, `spec.md`, `data-model.md`, `quickstart.md`, `contracts/`, and
   `tasks.md` artifacts kept in sync; missing files block work.
2. Interfaces MUST expose CLI/HTTP contracts that support both human-readable
   output and machine-parsable JSON; stderr carries diagnostics only.
3. Performance, security, and compliance budgets documented in `plan.md`
   translate into automated checks or TODOs with owners before coding begins.
4. Repository documentation and templates stay agent-agnostic; addenda belong
   in project docs, not vendor-specific scripts.

## Delivery Workflow & Quality Gates

1. **Phase 0 (Research)** - capture context, constraints, and success metrics,
   logging unanswered questions as blockers.
2. **Phase 1 (Design & Contracts)** - finalize plan, data models, quickstart,
   and contract tests for every user story; Constitution Check must pass via
   explicit references to each principle.
3. **Phase 2 (Foundations)** - build shared infrastructure listed in tasks;
   no story work proceeds until instrumentation and test harnesses exist.
4. **Phase 3+ (Story Execution)** - implement stories in priority order,
   keeping tests and documentation tied to story IDs; deliverable must run in
   isolation via `quickstart.md`.
5. **Review & Release** - reviewers verify Constitution compliance, ensure
   acceptance evidence is attached, and update change logs plus version tags.

## Governance

1. **Supremacy** - This constitution overrides conflicting team habits,
   checklists, or scripts. Deviations demand a written exception referencing
   impacted principles.
2. **Amendments** - Proposals require:
   - A written rationale citing principle(s) to alter.
   - Evidence from retrospectives, incidents, or metrics.
   - Updates to dependent templates/docs prior to ratification.
   Approval occurs via maintainer consensus recorded in PR history.
3. **Versioning Policy** - Semantic:
   - MAJOR when principles/governance become incompatible with prior work.
   - MINOR when adding or significantly expanding guidance.
   - PATCH for clarifications without behavioral change.
4. **Compliance Reviews** - Every plan, spec, and task doc must reference the
   latest constitution version and explicitly record pass/fail for each
   principle. Release reviews confirm observability artifacts and version bumps.
5. **Record Keeping** - Ratification and amendment dates stay in this file; the
   Sync Impact Report summarizes cross-file changes for traceability.

**Version**: 1.0.0 | **Ratified**: 2025-11-09 | **Last Amended**: 2025-11-09
