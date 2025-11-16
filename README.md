# Galarie

Galarie is a DB-free media browser that walks a mounted filesystem, parses filename tags, and serves a fast search/stream UI. A Rust (Axum) backend keeps the filesystem as the source of truth, caches metadata in JSON, and exposes `/api/v1` endpoints that power a React 18 + Vite SPA. The project ships with a devcontainer, an observability stack (OTel Collector, Prometheus, Loki, Tempo, Grafana), and Playwright E2E tests.

## Features

- **Filesystem-first catalog** – recursive scans turn filenames like `sunset+location-okinawa_rating-5.png` into structured tags and cached metadata.
- **Search, thumbnails, streaming** – REST APIs for media queries, thumbnail generation, and loopable video playback.
- **Favorites & tag filters** – SPA supports faceted filtering, favorites-driven slideshows, and invalid tag feedback.
- **Observability baked in** – OpenTelemetry traces/logs/metrics wired to the bundled collector + Grafana stack.
- **No external DBs** – deploy as a single container with media mounted read-only and cache stored in `/data/cache`.

## Repository Layout

```
backend/     # Rust backend (cargo workspace already configured)
frontend/    # React 18 + Vite SPA
sample-media/  # Versioned PNG/GIF/MP4 fixtures used for tests/demos
media/       # Empty mountpoint for your own media (gitignored)
.devcontainer/  # Devcontainer definition + observability docker-compose
Dockerfile   # Multi-stage build for production & devcontainer images
Makefile     # Convenience targets shared by backend/frontend
specs/       # Product, architecture, and task documents
```

## Quickstart (Devcontainer)

1. Install the [Dev Container CLI](https://github.com/devcontainers/cli) (or use VS Code Dev Containers) and Docker.
2. Clone the repo and prepare a media mount:

   ```bash
   git clone <repo> galarie
   cd galarie
   mkdir -p media            # host directory bind-mounted read-only
   devcontainer up --workspace-folder .
   devcontainer exec --workspace-folder . bash
   ```

3. Set runtime paths inside the container (adjust as needed):

   ```bash
   export GALARIE_MEDIA_ROOT=/workspace/media
   export GALARIE_CACHE_DIR=/workspace/.cache
   mkdir -p "$GALARIE_CACHE_DIR"
   ```

4. Seed sample assets (optional but useful):

   ```bash
   cp sample-media/* media/
   ```

Opening the devcontainer automatically launches the observability compose stack defined under `.devcontainer/observability/`. If you prefer to run it manually, execute `docker compose -f .devcontainer/observability/docker-compose.yaml up -d`.

## Running the Backend

Use the shared Make targets (they call `supervisord` within the devcontainer):

```bash
make backend/dev        # start Axum server
make backend/stop-dev   # stop it
make backend/test       # cargo test
make backend/lint       # cargo clippy -- -D warnings
make backend/fmt        # cargo fmt --check
```

To run directly via Cargo:

```bash
cd backend
cargo run -- \
  --media-root "$GALARIE_MEDIA_ROOT" \
  --cache-dir "$GALARIE_CACHE_DIR" \
  --listen 0.0.0.0:8080
```

Key env vars:

- `GALARIE_MEDIA_ROOT` – read-only mount for the filesystem crawl.
- `GALARIE_CACHE_DIR` – writable directory for `index.json` cache.
- `OTEL_EXPORTER_OTLP_ENDPOINT` – points to the collector (default `http://otel-collector:4317` inside docker-compose).
- `GALARIE_ENV`, `RUST_LOG`, `OTEL_SERVICE_NAME` for telemetry tuning (see `Dockerfile`).

## Running the Frontend

```bash
make frontend/install
make frontend/dev        # starts Vite dev server under supervisor
make frontend/stop-dev
make frontend/test       # Vitest
make frontend/e2e        # Playwright headless
make frontend/e2e-ui     # Playwright inspector
```

Manual operations:

```bash
cd frontend
npm install
npm run dev        # http://localhost:5173
npm run build
npm run preview
```

Set `VITE_API_BASE=http://localhost:8080/api/v1` in `frontend/.env` (dev) so the SPA talks to your backend.

## Observability Stack

The compose file in `.devcontainer/observability/` provides:

- OTel Collector (`localhost:4317`), Prometheus (`9090`), Loki (`3100`), Tempo (`3200`), Grafana (`3300`, admin/admin, anonymous enabled).
- Datasources are auto-provisioned, and the backend already exports traces/logs/metrics. Visit Grafana to inspect them while interacting with the UI.

## Sample Media & Ignored Content

`sample-media/README.md` documents three small fixtures generated with `ffmpeg`. Everything under `media/` is gitignored (except `.gitkeep` and the README), so you can safely mount personal libraries without risk of committing them.

## Testing Checklist

- `make backend/test`, `make backend/lint`, `make backend/fmt`
- `make frontend/test` (Vitest)
- `make frontend/e2e` (Playwright; run `make frontend/playwright-install` once per environment)
- API smoke tests: `curl -X POST http://localhost:8080/api/v1/index/rebuild -d '{"force":true}' -H 'Content-Type: application/json'` followed by `curl "http://localhost:8080/api/v1/media?page=1&pageSize=60"`

## Docker Builds & Releases

The top-level `Dockerfile` contains multi-stage builds:

- `backend-builder` (Rust)
- `frontend-builder` (Node/Vite)
- `prod-runtime` (slim Debian image that copies backend binary + frontend dist)
- `devcontainer` (extends backend-builder for VS Code / CLI devcontainers)

Build the production image locally:

```bash
docker build \
  --target prod-runtime \
  -t your-dockerhub-username/galarie:latest \
  .
```

Run it by mounting media/cache locations:

```bash
docker run --rm \
  -p 8080:8080 \
  -v "$PWD/media":/data/media:ro \
  -v "$PWD/.cache":/data/cache \
  your-dockerhub-username/galarie:latest
```

### GitHub Actions → Docker Hub

`/.github/workflows/dockerhub-publish.yml` builds the `prod-runtime` stage and pushes it whenever a Git tag is pushed to GitHub. Configure repository secrets:

- `DOCKERHUB_USERNAME`
- `DOCKERHUB_TOKEN` (Docker Hub access token/password with push rights)

Tagging `v0.1.0` results in `docker.io/<username>/galarie:v0.1.0`. The workflow does not publish `latest` automatically; add an extra metadata rule if desired.

## Contributing

1. Create a feature branch (`NNN-feature-name` to satisfy `.specify` scripts).
2. Keep OpenTelemetry wiring intact when touching backend code.
3. Run the Makefile lint/test targets for both backend and frontend.
4. Submit PRs with relevant specs/tasks referenced under `specs/galarie-media-platform/`.

For questions about architecture or roadmap, start with `specs/galarie-media-platform/plan.md` and `specs/galarie-media-platform/spec.md`.
