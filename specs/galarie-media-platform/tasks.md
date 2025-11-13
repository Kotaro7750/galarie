---

description: "Task list for Galarie media platform core feature"

---

# Tasks: Galarie Media Platform Core

**Input**: Design documents from `/specs/galarie-media-platform/`  
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md, contracts/
**Constitution Version**: 1.0.0 (cite in release notes)

**Tests**: Contract, integration, and critical unit tests are mandatory per Principle IV. Capture them before any implementation task and ensure they fail prior to code changes.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions
- Include rollback or feature-flag steps so each story can be disabled independently
- Reference logging/metrics work to satisfy Principle V

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Install devcontainer dependencies (Rust stable, Node 24 fallback, ffmpeg/gifsicle) via `.devcontainer` and verify `devcontainer up --workspace-folder .`.
- [x] T002 Create `media/` sample directory with fixture files + tags for local testing (document in `/specs/galarie-media-platform/quickstart.md`).
- [x] T003 [P] Configure shared `Makefile`/`justfile` in project root for backend/frontend commands with lint/test targets.

---

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T010 Scaffold backend Rust workspace `backend/` with Axum app, configuration loader, and OTLP exporter wiring.
- [x] T011 Implement filesystem watcher/indexer skeleton in `backend/src/indexer.rs` that walks `GALARIE_MEDIA_ROOT` and emits `MediaFile` structs (no cache persistence yet).
- [x] T012 Initialize JSON cache module `backend/src/cache/mod.rs` to persist/read `/workspace/.cache/index.json` with schema versioning + fallback rebuild logic.
- [x] T013 [P] Add thumbnail/streaming utilities: ffmpeg integration for video poster frames, gifsicle/image resizing in `backend/src/media/thumbnails.rs`.
- [x] T014 [P] Configure OpenTelemetry spans/logs/metrics in `backend/src/o11y/mod.rs` (search, rebuild, slideshow, video playback events) and expose `POST /api/v1/index/rebuild`.
- [ ] T014a [P] Add HTTP metrics middleware (OpenTelemetry) for all API endpoints to capture request count and latency. _(Deferred until after Phase 3 US1 per 2025-11-12 decision; observability impact accepted temporarily.)_
- [x] T015 Create shared error handling + REST response layer in `backend/src/api/mod.rs` (ErrorResponse type, consistent codes).
- [x] T016 Setup frontend skeleton `frontend/` (Vite + React + TypeScript + Tailwind) with environment config for API base URL and lint/test tooling (ESLint, Vitest).
- [x] T017 [P] Add Playwright E2E harness referencing Chrome and devcontainer instructions.
- [X] T018 Draft contract tests for each API endpoint in `backend/tests/contract/` (Rust integration) verifying failure first (run against stub responses).

**Checkpoint**: Backend API skeleton + cache + observability + frontend shell + failing contract tests ready.

---

## Phase 3: User Story 1 - Tag-Based Thumbnail Search (Priority: P1) 🎯 MVP

### Tests (write first)

- [X] T101 [P][US1] Contract test `backend/tests/contract/media_search.rs` verifying AND semantics, pagination, and error cases.
- [X] T102 [US1] Integration test `backend/tests/integration/search_cache.rs` ensures cache miss rebuilds and responds ≤1s for sample dataset.
- [X] T103 [P][US1] Frontend unit tests (Vitest) for search form logic + tag filter parsing (`frontend/src/hooks/useTagFilters.test.ts`).
<<<<<<< HEAD
- [X] T104 [P][US1] Contract test `backend/tests/contract/media_stream.rs` covering Range responses, inline disposition defaults, and validation errors.
- [X] T105 [US1] Integration test `backend/tests/integration/media_stream.rs` ensuring real files stream with correct MIME/ETag/404 handling.

### Implementation

- [X] T110 Implement tag parser + `Tag` struct normalization in `backend/src/tags/parser.rs`, including UTF-8 handling and invalid token warnings.
- [X] T111 Wire cache-backed search service in `backend/src/services/search.rs` applying `TagFilter` + inverted index.
- [X] T112 Expose `GET /api/v1/media` handler in `backend/src/api/search.rs` with query validation + error codes.
- [X] T113 Generate thumbnails on-demand/cache path management in `backend/src/media/thumbnails.rs`, returning CDN-safe URLs.
- [X] T114 [P] Build React search UI (`frontend/src/pages/SearchPage.tsx`) with tag filter inputs, multi-value attribute chips, results grid, and loading state.
- [X] T115 [P] Add SWR/TanStack Query data fetching service (`frontend/src/services/mediaClient.ts`) with error toasts + retry.
- [X] T116 Implement state persistence for search filters via `sessionStorage` (`frontend/src/hooks/usePersistedFilters.ts`).
- [ ] T117 Instrument search + streaming tracing/logging (span attributes: tags count, cache hit, bytes served, range info) and expose toggle to disable instrumentation for rollback.
- [ ] T118 Implement backend streaming handler (`backend/src/api/stream.rs`) that validates `disposition`, parses Range headers, detects MIME type, and streams from the media root.
- [ ] T119 Wire `/api/v1/media/{id}/stream` into the router with path traversal guards, telemetry counters, and cache-friendly headers (ETag, Accept-Ranges).
- [ ] T120 Update frontend media grid/detail components to open the streaming endpoint (inline playback/download) so US1 users can view originals immediately after search.

**Rollback**: Removing `frontend` search components + disabling `GET /api/v1/media` route returns system to baseline; cache JSON can be deleted safely.

---

## Phase 4: User Story 2 - Favorites Slideshow for Images/GIFs (Priority: P2)

### Tests (write first)

- [ ] T201 [P][US2] Frontend unit tests for favorites store + slideshow interval logic (`frontend/src/state/favoritesStore.test.ts`).
- [ ] T202 [US2] Playwright test covering search → add favorites → start slideshow → stop (state preserved).
- [ ] T203 [US2] Backend integration test ensuring slideshow static assets served correctly and instrumentation logs emitted.

### Implementation

- [ ] T210 Implement client-side favorites store with Zustand/Context (`frontend/src/state/favoritesStore.ts`) persisted in `sessionStorage`.
- [ ] T211 Build favorites panel UI (`frontend/src/components/FavoritesSidebar.tsx`) showing order, remove buttons, and count.
- [ ] T212 Create slideshow player `frontend/src/components/SlideshowPlayer.tsx` with interval input, fixed ordering, and Fullscreen API integration.
- [ ] T213 Ensure GIF playback uses native animation (no extra processing) and preloads via `<img>` to avoid flicker.
- [ ] T214 Add observability: emit browser telemetry events (console + optional log endpoint stub) when slideshow starts/stops.
- [ ] T215 Add backend feature flag env (`ENABLE_SLIDESHOW_STATIC=true`) to allow rollback by disabling static asset route.

**Rollback**: Disable slideshow flag + hide favorites UI; delete client `sessionStorage` keys.

---

## Phase 5: User Story 3 - Video Loop & A-B Repeat (Priority: P3)

### Tests (write first)

- [ ] T301 [P][US3] Frontend unit test for video loop state machine (`frontend/src/hooks/useVideoLoop.test.ts`).
- [ ] T302 [US3] Playwright scenario verifying A/B markers, loop toggle, and state persistence within session.
- [ ] T303 [US3] Telemetry test (contract/integration) ensuring video loop events emit spans/logs with mediaId + marker metadata.

### Implementation

- [ ] T310 Implement video player component `frontend/src/components/VideoPlayer.tsx` with controls for loop, A/B set/clear, and fullscreen support.
- [ ] T311 Add hook `useVideoLoop.ts` managing state, storing markers per mediaId in memory/session storage.
- [ ] T312 Implement telemetry/logging pipeline for video loop interactions (frontend event emitter or log endpoint) feeding OpenTelemetry spans/metrics.
- [ ] T313 Provide UI affordance to exit video mode without resetting search/favorites (shared state remains intact).
- [ ] T314 Document keyboard shortcuts (e.g., “A” sets marker A) in UI tooltip.

**Rollback**: Toggle feature flag `ENABLE_VIDEO_LOOP=false` to hide loop controls; streaming endpoint remains for basic playback.

---

## Phase N: Polish & Cross-Cutting Concerns

- [ ] T901 Documentation refresh: update `/docs/` (or README) with setup, telemetry, troubleshooting.
- [ ] T902 Add structured logging sinks + Grafana dashboard samples referencing OTel collector.
- [ ] T903 [P] Performance profiling: run `cargo flamegraph` and React Profiler to ensure search/slideshow meet targets.
- [ ] T904 Security review: verify path sanitization, CORS, and content-type headers.
- [ ] T905 [P] Additional unit/integration tests for edge cases (invalid tags, missing files, large GIFs).
- [ ] T906 Run quickstart validation end-to-end and capture demo screenshots/gifs.
