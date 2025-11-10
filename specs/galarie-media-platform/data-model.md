# Data Model: Galarie Media Platform

## Overview

The system keeps the filesystem as the source of truth and derives in-memory/JSON indices for fast tag search. The following logical entities drive the design:

1. **MediaFile** – Parsed metadata for each file the backend can serve.  
2. **Tag** – Normalized representation of both simple and key/value tags.  
3. **TagFilter** – Search criteria provided by the frontend.  
4. **FavoriteSet** – Client-side structure (persisted in browser storage) representing a slideshow queue.  
5. **IndexCache** – On-disk JSON snapshot storing MediaFile entries to avoid reparsing on every request.

## Entities

### MediaFile

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Stable identifier (e.g., SHA-1 of relative path). |
| `relativePath` | string | Path relative to mounted root (`media/photos/foo.jpg`). |
| `mediaType` | enum(`image`,`gif`,`video`,`audio`,`pdf`) | Primary type, extensible for future formats. |
| `tags` | Tag[] | Collection of normalized tags (simple or key/value). |
| `attributes` | map<string,string> | Convenience map for quick lookup (`rating:5`). |
| `filesize` | number | Bytes (from FS stat). |
| `dimensions` | `{ width: number, height: number }?` | For image/GIF/PDF pages when known. |
| `durationMs` | number? | Video/audio length. |
| `thumbnailPath` | string | Path to cached thumbnail (relative to working dir). |
| `hash` | string? | Optional checksum to detect changes. |
| `indexedAt` | string (ISO datetime) | Last index timestamp. |

### TagFilter

| Field | Type | Description |
|-------|------|-------------|
| `requiredTags` | string[] | Simple tags that MUST exist. |
| `attributeFilters` | map<string,string[]> | AND semantics; each key must match one of the listed values. |
| `excludeTags` | string[] (future) | Reserved for potential NOT support. |
| `page` | number | 1-based page index. |
| `pageSize` | number | Result size per page. |

### FavoriteSet (Client-side)

| Field | Type | Description |
|-------|------|-------------|
| `items` | MediaFile reference[] | Ordered list of selected media IDs. |
| `intervalMs` | number | Slideshow interval. |
| `createdAt` | string | ISO timestamp. |

### IndexCache

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Schema version (`1.0.0`). |
| `generatedAt` | string | ISO timestamp. |
| `media` | MediaFile[] | Flattened array for fast lookup. |
| `tagLookup` | map<string,string[]> | Inverted index (tag -> media IDs). |
| `attributeLookup` | map<string,map<string,string[]>> | Key -> value -> media IDs. |

## Relationships & Flows

1. **Filesystem → MediaFile/Tag**: On index rebuild, the backend walks the mounted directory, parses filename tokens into Tag objects, inspects file metadata (dimensions/duration), and produces MediaFile instances stored in IndexCache.
2. **TagFilter → MediaFile[]**: Incoming search requests build a TagFilter that the backend applies against the IndexCache, returning paginated MediaFile summaries.
3. **MediaFile → Thumbnail/Stream**: Each MediaFile points to thumbnail and original stream endpoints derived from `relativePath`.
4. **FavoriteSet → Slideshow**: Managed client-side; backend does not persist favorites, but receives `mediaId` sequences when streaming.
5. **Observability Signals**: OpenTelemetry spans/logs/metrics accompany indexing, search, slideshow, and video playback operations. Required attributes/metrics are documented separately (e.g., `otel.md`) rather than modeled as entities because OTel schemas already define the underlying data format.

## Notes

- **Extensibility**: `mediaType` includes `audio`/`pdf` to accommodate future formats without schema changes.
- **Localization**: Tags remain UTF-8 strings; filesystem encoding issues are handled during parsing.
- **Consistency**: `id` is derived from normalized relative paths so that renaming files regenerates predictable identifiers, causing cache updates.
- **Security**: Only paths under the mounted root are allowed; `relativePath` is sanitized to prevent traversal.
### Tag

| Field | Type | Description |
|-------|------|-------------|
| `rawToken` | string | Original segment from filename (`travel`, `rating-5`). |
| `type` | enum(`simple`,`kv`) | Distinguishes plain tags vs key/value. |
| `name` | string | Tag name (e.g., `rating`). |
| `value` | string? | Value for key/value tags; null for simple tags. |
| `normalized` | string | Lowercase slug used for indexing (`rating=5`). |
