SHELL := /bin/bash

BACKEND_DIR ?= backend
FRONTEND_DIR ?= frontend
FRONTEND_PM ?= npm

.PHONY: help \
	backend/test backend/lint backend/fmt \
	frontend/install frontend/dev frontend/test frontend/lint frontend/build \
	frontend/e2e frontend/e2e-ui frontend/playwright-install

help:
	@printf "Available targets:\n"
	@printf "  make backend/dev     # start backend dev server (requires %s)\n" "$(BACKEND_DIR)"
	@printf "  make backend/stop-dev# stop backend dev server (requires %s"
	@printf "  make backend/test    # cargo test (requires %s)\n" "$(BACKEND_DIR)"
	@printf "  make backend/lint    # cargo clippy -- -D warnings\n"
	@printf "  make backend/fmt     # cargo fmt --check\n"
	@printf "  make frontend/install# %s install in %s\n" "$(FRONTEND_PM)" "$(FRONTEND_DIR)"
	@printf "  make frontend/dev    # %s run dev\n" "$(FRONTEND_PM)"
	@printf "  make frontend/stop-dev # %s stop frontend dev server (requires %s)\n" "$(FRONTEND_PM)" "$(FRONTEND_DIR)"
	@printf "  make frontend/test   # %s run test\n" "$(FRONTEND_PM)"
	@printf "  make frontend/lint   # %s run lint\n" "$(FRONTEND_PM)"
	@printf "  make frontend/build  # %s run build\n" "$(FRONTEND_PM)"
	@printf "  make frontend/e2e    # %s run test:e2e (Playwright)\n" "$(FRONTEND_PM)"
	@printf "  make frontend/e2e-ui # %s run test:e2e:ui (Playwright UI)\n" "$(FRONTEND_PM)"
	@printf "  make frontend/playwright-install # %s run playwright:install\n" "$(FRONTEND_PM)"

backend/stop-dev:
	@if [ ! -d "$(BACKEND_DIR)" ]; then \
		echo "Missing $(BACKEND_DIR)/. Scaffold backend before running backend/stop-dev." >&2; \
		exit 1; \
	fi
	sudo supervisorctl stop backend

backend/dev:
	@if [ ! -d "$(BACKEND_DIR)" ]; then \
		echo "Missing $(BACKEND_DIR)/. Scaffold backend before running backend/dev." >&2 \
		exit 1; \
	fi
	sudo supervisorctl start backend

backend/test:
	@if [ ! -d "$(BACKEND_DIR)" ]; then \
		echo "Missing $(BACKEND_DIR)/. Scaffold backend before running backend/test." >&2; \
		exit 1; \
	fi
	cd "$(BACKEND_DIR)" && cargo test

backend/lint:
	@if [ ! -d "$(BACKEND_DIR)" ]; then \
		echo "Missing $(BACKEND_DIR)/. Scaffold backend before running backend/lint." >&2; \
		exit 1; \
	fi
	cd "$(BACKEND_DIR)" && cargo clippy --all-targets --all-features -- -D warnings

backend/fmt:
	@if [ ! -d "$(BACKEND_DIR)" ]; then \
		echo "Missing $(BACKEND_DIR)/. Scaffold backend before running backend/fmt." >&2; \
		exit 1; \
	fi
	cd "$(BACKEND_DIR)" && cargo fmt --all --check

frontend/install:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/install." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) install

frontend/stop-dev:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/stop-dev." >&2; \
		exit 1; \
	fi
	sudo supervisorctl stop frontend

frontend/dev:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/dev." >&2; \
		exit 1; \
	fi
	sudo supervisorctl start frontend

frontend/test:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/test." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run test

frontend/lint:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/lint." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run lint

frontend/build:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/build." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run build

frontend/e2e:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/e2e." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run test:e2e

frontend/e2e-ui:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/e2e-ui." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run test:e2e:ui

frontend/playwright-install:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/playwright-install." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run playwright:install
