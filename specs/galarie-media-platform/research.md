# Project Purpose Brief

## 1. Mission Statement
- **Primary user**: The repository owner and other enthusiasts who store multimedia (images, GIFs, videos) locally with tag-embedded filenames.
- **Problem**: File-name-based search only supports simple tag queries and offers no curated viewing experiences (favorites, slideshows, looped playback).
- **Value**: Provide a DB-free, file-system-first web experience that delivers powerful tag filtering and enjoyable viewing workflows while keeping files editable via standard filesystem operations.

## 2. Desired Outcomes (Principle V)
- **Search performance**: Return results from thousands of files in ≤1s via cached tag indices.
- **Workflow efficiency**: Allow building a slideshow-ready favorite list within three clicks from search results.
- **Operational visibility (MUST)**: Export OpenTelemetry traces, metrics, and logs for indexing, search, slideshow, and video playback flows; profiles are OPTIONAL but preferred when feasible.

## 3. Scope Boundaries
- **Must include**: Tag parsing for `tag` and `key:value` formats, AND-only multi-filter search, thumbnail grid, favorites list, image/GIF slideshow, video A-B loop.
- **Must exclude**: External databases, cloud-specific services, multi-user auth, remote storage sync, advanced tag hierarchies.

## 4. Core User Journeys
1. **US1 (P1)** – Perform AND-based tag searches and view thumbnails; cache is regenerated on demand.
2. **US2 (P1/P2)** – Select favorites from search results and run a configurable-interval slideshow (fixed order, infinite loop).
3. **US3 (P2/P3)** – Watch videos with loop and A-B repeat controls without losing search/favorite context.

## 5. Research & Evidence
- **File metadata**: Tags encoded in filenames (`tag1_tag2_key-value.ext`) with UTF-8 support; consistent delimiter usage required.
- **Existing tooling**: Prior Rust tooling can extract images from PDF pages, hinting at future format expansion.
- **Scale**: A few thousand files; caching tag parses (JSON, in-memory) keeps latency low without a DB.
- **Risks**:
  - Ensuring consistent parsing despite filesystem limitations on characters.
  - Generating thumbnails for large GIFs/videos may be CPU intensive inside containers.
  - Maintaining state (favorites, search filters) client-side until browser close; no persistent sessions.

## 6. Interface Expectations
- **Backend**: Go or Rust API server running in Kubernetes, with host filesystem mounted read-only except for cache directories.
- **Frontend**: React SPA (Vite) calling JSON APIs for search, favorites, slideshow control.
- **Data exchange**: JSON payloads returning file metadata, tag breakdowns, and playable URLs (served via the API).

## 7. Test & Compliance Hooks
- **Unit/contract tests**: Tag parser, search filters, cache invalidation.
- **Integration tests**: End-to-end search → thumbnail → favorite selection flows.
- **Operational tests**: Ensure structured logs emit indexing status and slideshow playback markers for observability.
