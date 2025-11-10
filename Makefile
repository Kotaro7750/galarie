SHELL := /bin/bash

BACKEND_DIR ?= backend
FRONTEND_DIR ?= frontend
FRONTEND_PM ?= npm

.PHONY: help \
	backend/test backend/lint backend/fmt \
	frontend/install frontend/dev frontend/test frontend/lint frontend/build

help:
	@printf "Available targets:\n"
	@printf "  make backend/test    # cargo test (requires %s)\n" "$(BACKEND_DIR)"
	@printf "  make backend/lint    # cargo clippy -- -D warnings\n"
	@printf "  make backend/fmt     # cargo fmt --check\n"
	@printf "  make frontend/install# %s install in %s\n" "$(FRONTEND_PM)" "$(FRONTEND_DIR)"
	@printf "  make frontend/dev    # %s run dev\n" "$(FRONTEND_PM)"
	@printf "  make frontend/test   # %s run test\n" "$(FRONTEND_PM)"
	@printf "  make frontend/lint   # %s run lint\n" "$(FRONTEND_PM)"
	@printf "  make frontend/build  # %s run build\n" "$(FRONTEND_PM)"

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

frontend/dev:
	@if [ ! -d "$(FRONTEND_DIR)" ]; then \
		echo "Missing $(FRONTEND_DIR)/. Scaffold frontend before running frontend/dev." >&2; \
		exit 1; \
	fi
	cd "$(FRONTEND_DIR)" && $(FRONTEND_PM) run dev

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
