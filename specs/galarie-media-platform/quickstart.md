# Quickstart: Galarie Media Platform

> Goal: run the backend API (Rust/Go) and React SPA locally, index a media directory, and verify search/slideshow/video flows.

## 1. Prerequisites

- Dev Container CLI (preferred) or VS Code Dev Containers; once configured, all toolchains (Rust latest stable + Node 24.x when available, ffmpeg/gifsicle) live inside the devcontainer image.  
- Note: The devcontainer attempts to install Node 24.x automatically and falls back to the newest NodeSource release until 24.x packages are published.
- Docker or compatible runtime with access to your media directory (mount read-only into the devcontainer).  
- Optional: OpenTelemetry Collector (can run as another container alongside the devcontainer, exposing `4317`).

## 2. Clone & Prepare

```bash
git clone <repo> galarie
cd galarie
mkdir -p media            # host directory that devcontainer bind-mounts read-only
```

Start the devcontainer once `.devcontainer/` is added:

```bash
devcontainer up --workspace-folder .
devcontainer exec --workspace-folder . bash
```

## 3. Configure Media Mount

Set environment variables (example inside devcontainer):

```bash
export GALARIE_MEDIA_ROOT=/workspace/media      # bind-mounted host path
export GALARIE_CACHE_DIR=/workspace/.cache      # inside container working dir
export GALARIE_PORT=8080
mkdir -p "$GALARIE_CACHE_DIR"
```

## 4. Seed Sample Media

Copy the bundled fixtures into your local `media/` mount:

```bash
cp sample-media/* media/
```

- PNG/GIF/MP4 assets are versioned in `sample-media/`, so no generation step is required.
- Feel free to copy your own files into `media/`; git ignores everything except `README.md` and `.gitkeep`.

## 5. Backend Setup (Rust example)

```bash
cd backend
cargo install just # optional task runner
cargo test
cargo run -- \
  --media-root "$GALARIE_MEDIA_ROOT" \
  --cache-dir "$GALARIE_CACHE_DIR" \
  --listen 0.0.0.0:$GALARIE_PORT
```

Or simply invoke the shared Makefile targets from the repo root:

```bash
make backend/test
make backend/lint
```

Run `make help` anytime to list all available targets.

Go variant:

```bash
cd backend
go test ./...
go run ./cmd/server \
  --media-root "$GALARIE_MEDIA_ROOT" \
  --cache-dir "$GALARIE_CACHE_DIR" \
  --listen :$GALARIE_PORT
```

The API serves:
- `GET /api/v1/media`
- `GET /api/v1/media/{id}/thumbnail`
- `GET /api/v1/media/{id}/stream`
- `POST /api/v1/index/rebuild`

## 6. Frontend Setup

```bash
cd frontend
npm install
npm run dev   # http://localhost:5173 by default
# or build & serve
npm run build
npm run preview
```

Makefile wrappers from the repo root:

```bash
make frontend/install
make frontend/dev
make frontend/test
```

Configure `.env`:

```
VITE_API_BASE=http://localhost:8080/api/v1
```

## 7. Index Rebuild & Verification

Trigger cache rebuild:

```bash
curl -X POST http://localhost:8080/api/v1/index/rebuild -d '{"force":true}' -H 'Content-Type: application/json'
```

Check search endpoint:

```bash
curl "http://localhost:8080/api/v1/media?tags=cat,sunset&attributes[rating]=5"
```

## 8. UI Workflow Test

1. Open the frontend (dev server or bundled build).  
2. Perform tag search → thumbnails appear rapidly (<=1s).  
3. Select favorites → start slideshow → confirm interval/loop.  
4. Pick a video → use loop/A-B controls; state persists until reload.  
5. Observe OpenTelemetry traces/logs (if collector container is running).

## 9. OpenAPI Docs

To preview API docs (optional):

```bash
npx -y redoc-cli serve specs/galarie-media-platform/contracts/openapi.yaml
```

Browse `http://localhost:8080` (default). Use `Ctrl+C` to stop.

## 10. Troubleshooting

- Ensure media path is mounted read-only; backend logs warn if missing.  
- Delete cache (`rm -rf $GALARIE_CACHE_DIR/index.json`) if tags change.  
- Install ffmpeg/gifsicle in container to avoid thumbnail failures.  
- Confirm OTel collector endpoint (`OTEL_EXPORTER_OTLP_ENDPOINT`) is reachable; otherwise disable instrumentation via env flag.  
- Use `curl /healthz` (to be implemented) to verify backend readiness.
