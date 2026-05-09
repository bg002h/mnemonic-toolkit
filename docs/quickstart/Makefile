# mnemonic-toolkit/docs/quickstart/Makefile
#
# Build the m-format Quick Start guide to markdown and PDF.
#
# Targets:
#   make md            — concatenated markdown to build/m-format-quickstart.md
#   make pdf           — typeset PDF to build/m-format-quickstart.pdf (needs pandoc + xelatex + mermaid-filter locally)
#   make pdf-docker    — same as `make pdf`, inside Dockerfile.build (pinned, reproducible)
#   make figures-cache — pre-render mermaid blocks to figures/cache/ for MERMAID_FILTER=skip mode
#   make verify-examples — re-run worked-example transcripts against locally-built CLIs
#   make lint          — markdownlint + cspell + lychee (3-of-6 from manual; no glossary/flag/index checks)
#   make clean         — remove build/ and LaTeX byproducts
#   make help          — print this help
#
# Variables (override on the command line or in CI):
#   MNEMONIC_BIN, MD_BIN, MS_BIN — CLI invocation strings used by verify-examples.
#   MERMAID_FILTER  — 'on' (default) runs mermaid-filter; 'skip' uses figures/cache/ snapshots only.
#   PANDOC, XELATEX — tool paths (override if not on $PATH).
#   DOCKER_IMAGE — name to tag the build image as (default mnemonic-quickstart-build:latest).

# ---------------------------------------------------------------------------
# Repo-relative paths
# ---------------------------------------------------------------------------
QUICKSTART_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TOOLKIT_ROOT   := $(abspath $(QUICKSTART_DIR)/../..)
WORKSPACE_ROOT := $(abspath $(TOOLKIT_ROOT)/..)

SRC_DIR        := $(QUICKSTART_DIR)/src
PANDOC_DIR     := $(QUICKSTART_DIR)/pandoc
FILTERS_DIR    := $(PANDOC_DIR)/filters
TEMPLATES_DIR  := $(PANDOC_DIR)/templates
TESTS_DIR      := $(QUICKSTART_DIR)/tests
BUILD_DIR      := $(QUICKSTART_DIR)/build
TRANSCRIPTS    := $(QUICKSTART_DIR)/transcripts

# ---------------------------------------------------------------------------
# CLI binary invocation strings (overrideable; CI overrides to pre-built bins)
# ---------------------------------------------------------------------------
MNEMONIC_BIN ?= cargo run --quiet --manifest-path $(TOOLKIT_ROOT)/Cargo.toml -p mnemonic-toolkit --bin mnemonic --
MD_BIN       ?= cargo run --quiet --manifest-path $(WORKSPACE_ROOT)/descriptor-mnemonic/Cargo.toml -p md-cli --bin md --
MS_BIN       ?= cargo run --quiet --manifest-path $(WORKSPACE_ROOT)/mnemonic-secret/Cargo.toml -p ms-cli --bin ms --

# ---------------------------------------------------------------------------
# Tools (override with PANDOC=/path/to/pandoc etc.)
# ---------------------------------------------------------------------------
PANDOC      ?= pandoc
XELATEX     ?= xelatex
DOCKER      ?= docker
DOCKER_IMAGE ?= mnemonic-quickstart-build:latest

# ---------------------------------------------------------------------------
# Filter / build mode
# ---------------------------------------------------------------------------
MERMAID_FILTER ?= on

# ---------------------------------------------------------------------------
# Source enumeration
# ---------------------------------------------------------------------------
# All source files, lexicographic order. Use `find -print` rather than $(shell)
# bash globbing so the recursive sort is deterministic and stable.
ALL_SRC := $(shell find $(SRC_DIR) -type f -name '*.md' | LC_ALL=C sort)
# `make md` excludes 99-build-banner.md; `make pdf` includes it (last).
MD_SRC  := $(shell find $(SRC_DIR) -type f -name '*.md' ! -name '99-build-banner.md' | LC_ALL=C sort)
PDF_SRC := $(MD_SRC) $(SRC_DIR)/99-build-banner.md

# ---------------------------------------------------------------------------
# Filter args
# ---------------------------------------------------------------------------
# `make md` runs strip-latex + primer-box(md mode); `make pdf*` runs primer-box(pdf mode) only.
MD_FILTER_ARGS  := --lua-filter $(FILTERS_DIR)/strip-latex-from-md.lua --lua-filter $(FILTERS_DIR)/primer-box.lua
PDF_FILTER_ARGS := --lua-filter $(FILTERS_DIR)/primer-box.lua

ifeq ($(MERMAID_FILTER),on)
	PDF_FILTER_ARGS += --filter mermaid-filter
endif

PANDOC_METADATA := --metadata-file=$(PANDOC_DIR)/metadata.yaml

# ---------------------------------------------------------------------------
# Default target
# ---------------------------------------------------------------------------
.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^# ?(make |  )' $(MAKEFILE_LIST) | sed 's/^# \?//'

# ---------------------------------------------------------------------------
# Build directory
# ---------------------------------------------------------------------------
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# ---------------------------------------------------------------------------
# Markdown render path: --to gfm (GitHub-Flavored Markdown)
# ---------------------------------------------------------------------------
.PHONY: md
md: $(BUILD_DIR)/m-format-quickstart.md

$(BUILD_DIR)/m-format-quickstart.md: $(MD_SRC) $(FILTERS_DIR)/strip-latex-from-md.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to gfm \
		--standalone \
		--toc \
		--toc-depth=3 \
		$(MD_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--output $@ \
		$(MD_SRC)

# ---------------------------------------------------------------------------
# PDF render path: pandoc --to latex → xelatex → xelatex
# (No makeindex: QuickStart has no \index{} markers.)
# ---------------------------------------------------------------------------
.PHONY: pdf
pdf: $(BUILD_DIR)/m-format-quickstart.pdf

# Regenerate the build banner each PDF build so $(GIT_SHA) / $(BUILD_DATE) are fresh.
GIT_SHA    ?= $(shell git -C $(TOOLKIT_ROOT) rev-parse --short HEAD 2>/dev/null || echo unknown)
BUILD_DATE ?= $(shell date -u +%Y-%m-%dT%H:%M:%SZ)

$(SRC_DIR)/99-build-banner.md: FORCE
	@printf '%s\n' \
		'# Build banner' \
		'' \
		'Built from commit `$(GIT_SHA)` on `$(BUILD_DATE)`.' \
		'' \
		'_This page is included in the PDF render only._' > $@

.PHONY: FORCE
FORCE:

$(BUILD_DIR)/m-format-quickstart.tex: $(PDF_SRC) $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml $(PANDOC_DIR)/preamble.tex | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to latex \
		--standalone \
		--toc \
		--toc-depth=3 \
		--top-level-division=chapter \
		-H $(PANDOC_DIR)/preamble.tex \
		$(PDF_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--output $@ \
		$(PDF_SRC)

$(BUILD_DIR)/m-format-quickstart.pdf: $(BUILD_DIR)/m-format-quickstart.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-quickstart.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-quickstart.tex

# ---------------------------------------------------------------------------
# Dockerized PDF build (canonical)
# ---------------------------------------------------------------------------
.PHONY: pdf-docker docker-image
docker-image:
	$(DOCKER) build -t $(DOCKER_IMAGE) -f $(QUICKSTART_DIR)/Dockerfile.build $(QUICKSTART_DIR)

pdf-docker: docker-image
	$(DOCKER) run --rm \
		-u $$(id -u):$$(id -g) \
		-v $(TOOLKIT_ROOT):/work \
		-w /work/docs/quickstart \
		$(DOCKER_IMAGE) \
		make pdf MERMAID_FILTER=$(MERMAID_FILTER)

# ---------------------------------------------------------------------------
# Pre-rendered mermaid figures cache (offline build fallback)
# ---------------------------------------------------------------------------
.PHONY: figures-cache
figures-cache:
	@echo "figures-cache: not yet implemented (see FOLLOWUPS.md). Run with MERMAID_FILTER=on for live rendering."
	@false

# ---------------------------------------------------------------------------
# Worked-example regeneration / verification
# ---------------------------------------------------------------------------
.PHONY: verify-examples
verify-examples:
	@bash $(TESTS_DIR)/verify-examples.sh \
		MNEMONIC_BIN="$(MNEMONIC_BIN)" \
		MD_BIN="$(MD_BIN)" \
		MS_BIN="$(MS_BIN)" \
		TRANSCRIPTS="$(TRANSCRIPTS)"

# ---------------------------------------------------------------------------
# Lint
# ---------------------------------------------------------------------------
.PHONY: lint
lint:
	@bash $(TESTS_DIR)/lint.sh \
		SRC_DIR="$(SRC_DIR)" \
		TESTS_DIR="$(TESTS_DIR)" \
		MNEMONIC_BIN="$(MNEMONIC_BIN)" \
		MD_BIN="$(MD_BIN)" \
		MS_BIN="$(MS_BIN)"

# ---------------------------------------------------------------------------
# Release-asset attach (called from CI or by hand on a tagged build)
# ---------------------------------------------------------------------------
.PHONY: release-attach
release-attach: pdf-docker
ifndef VERSION
	$(error VERSION must be set, e.g. `make release-attach VERSION=quickstart-v0.1.0`)
endif
	gh release upload $(VERSION) \
		$(BUILD_DIR)/m-format-quickstart.pdf \
		--clobber

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(SRC_DIR)/99-build-banner.md
