# Implementation Plan: Galarie Media Platform Core

**Branch**: `[001-galarie-media-platform]` | **Date**: 2025-11-09 | **Spec**: `specs/galarie-media-platform/spec.md`
**Input**: Feature specification from `/specs/galarie-media-platform/spec.md`
**Constitution Version**: 1.0.0 (update if constitution changes)

## Summary

Deliver a DB-free multimedia viewer that mounts the local filesystem in a Kubernetes pod, parses tag metadata from filenames, exposes REST APIs for search/streaming, and serves a React SPA enabling advanced tag filters, favorites-driven slideshows, and video loop controls with OpenTelemetry instrumentation.

## Technical Context

**Language/Version**: Backend Rust (latest stable release, currently 1.82) or Go 1.22.x (latest stable) pending spike outcome; Frontend React 18 + Vite + TypeScript running on Node 24.x (devcontainer attempts 24 and temporarily falls back to newest NodeSource release until 24.x GA).  
**Primary Dependencies**:  
- Backend: Axum/Tonic (Rust) or Chi/Fiber (Go), Tokio or Go stdlib, OpenTelemetry SDK, image/gifsicle/ffmpeg bindings for thumbnails.  
- Frontend: React, TanStack Query, Tailwind CSS, Fullscreen API utilities.  
**Storage**: No DB. Uses mounted filesystem + temporary JSON cache (`/app/cache/index.json`) inside the container working directory, regenerated via API.  
**Testing**: Backend - cargo test / go test, contract tests via backend integration suites; Frontend - Vitest + Playwright for E2E.  
**Target Platform**: Self-hosted Kubernetes cluster (amd64).  
**Project Type**: Web app with separate backend/frontend directories.  
**Performance Goals**: Search latency <=1s for 2k files; thumbnail generation pipeline <200ms per image with caching; streaming honors Range requests.  
**Constraints**:  
- Filesystem must remain source of truth; cache rebuild must tolerate filename edits.  
- No reliance on cloud-managed services; container images must run offline.  
- Observability via OpenTelemetry (traces, logs, metrics; profiling optional).  
- Local development environment standardized via Dev Container CLI setup sharing the same toolchains (Rust/Go/Node) and filesystem mounts as production containers, without requiring VS Code.  
**Scale/Scope**: Initial dataset ~2k files, future-proof to ~10k with incremental indexing.

コメント
* いずれのツールを使うにしても、最新の安定版を使用するようにしてください。
* JSON一時キャッシュのパスは、コンテナで利用することを想定しているためコンテナのworking directory内部に配置するようにしてください

## Constitution Check

| Principle | Evidence to capture | Status (PASS/FAIL + link) |
|-----------|--------------------|---------------------------|
| Principle I - Independently Valuable Slices | US1 search, US2 slideshow, US3 video loop documented in `spec.md` with acceptance + rollback. | PASS (`spec.md#user-story-1` etc.) |
| Principle II - Research-Led Planning | Purpose, constraints, risks captured in `research.md` §1-7; unresolved risk: thumbnail cost for large media. | PASS (`research.md`) |
| Principle III - Contract-First Interfaces | REST endpoints defined in `spec.md` contract section; `contracts/` will include OpenAPI + contract tests before coding. | PASS (pending `contracts/openapi.yaml`) |
| Principle IV - Test-Gated Implementation | Backend contract/integration tests (Rust `tests/contract/*.rs` or Go equivalents) plus Playwright UI suites will be authored first; plan mandates failing tests before implementation. | PASS (tests to be authored Phase 1) |
| Principle V - Operational Transparency & Versioning | OpenTelemetry pipeline planned; semver via `/api/v1` namespace; logs/metrics/traces enumerated in `research.md` Desired Outcomes. | PASS |

## Project Structure

### Documentation (this feature)

```text
specs/galarie-media-platform/
├── plan.md
├── research.md
├── data-model.md          # TBD
├── quickstart.md          # TBD
├── contracts/
│   └── openapi.yaml       # Draft contract + schemas
└── tasks.md               # Generated later
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── main.rs (or main.go)
│   ├── media/
│   ├── tags/
│   ├── api/
│   └── o11y/
└── tests/
    ├── contract/
    ├── integration/
    └── unit/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   ├── hooks/
│   └── services/
└── tests/
    ├── unit/
    └── e2e/
```

**Structure Decision**: Separate backend (Rust/Go) and frontend (React) projects allow independent builds, align with container deployment, and reflect Principle I by isolating deliverables. Final deployment will bundle the frontend’s static build artifacts into the backend container image (served via the API pod) while keeping codebases independent for development. 

コメント
* バックエンドとフロントエンドのディレクトリを分けること自体は賛成なのですが、最終的にフロントエンドの成果物をバックエンドのコンテナと組み合わせるなども考えられます。その点についてはRv中で議論したいです。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Dual-language stack (Rust/Go + JS) | Needed to deliver SPA UX plus performant filesystem API. | Single-stack (e.g., full Node) would complicate filesystem performance and limit systems-level experimentation. |
