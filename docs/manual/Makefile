# mnemonic-toolkit/docs/manual/Makefile
#
# Build the m-format constellation user manual to markdown and PDF.
#
# Targets:
#   make md            — concatenated markdown to build/m-format-manual.md
#   make pdf           — typeset PDF to build/m-format-manual.pdf (needs pandoc + xelatex; consumes figures/cache/)
#   make pdf-docker    — same as `make pdf`, inside Dockerfile.build (pinned, reproducible)
#   make figures-cache — pre-render mermaid blocks to figures/cache/ (rare; needs mmdc + chromium locally)
#   make verify-examples — re-run worked-example transcripts against locally-built CLIs
#   make audit         — `lint` + `verify-examples` umbrella (CI runs this)
#   make lint          — markdownlint + cspell + lychee + flag-coverage + glossary-coverage + index bidirectional
#   make clean         — remove build/ and LaTeX byproducts
#   make help          — print this help
#
# Variables (override on the command line or in CI):
#   MNEMONIC_BIN, MD_BIN, MS_BIN, MK_BIN — CLI invocation strings used by verify-examples and flag-coverage.
#   MERMAID_FILTER  — 'skip' (default) consumes figures/cache/ snapshots; 'on' runs mermaid-filter
#                      (needs chromium + mermaid-filter locally — rare, only for cache regen review).
#   PANDOC, XELATEX, MAKEINDEX — tool paths (override if not on $PATH).
#   DOCKER_IMAGE — name to tag the build image as (default mnemonic-manual-build:latest).

# ---------------------------------------------------------------------------
# Repo-relative paths
# ---------------------------------------------------------------------------
MANUAL_DIR     := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TOOLKIT_ROOT   := $(abspath $(MANUAL_DIR)/../..)
WORKSPACE_ROOT := $(abspath $(TOOLKIT_ROOT)/..)

SRC_DIR        := $(MANUAL_DIR)/src
PANDOC_DIR     := $(MANUAL_DIR)/pandoc
FILTERS_DIR    := $(PANDOC_DIR)/filters
TEMPLATES_DIR  := $(PANDOC_DIR)/templates
FIGURES_DIR    := $(MANUAL_DIR)/figures
TESTS_DIR      := $(MANUAL_DIR)/tests
BUILD_DIR      := $(MANUAL_DIR)/build
TRANSCRIPTS    := $(MANUAL_DIR)/transcripts

# ---------------------------------------------------------------------------
# CLI binary invocation strings (overrideable; CI overrides to pre-built bins)
# ---------------------------------------------------------------------------
MNEMONIC_BIN ?= cargo run --quiet --manifest-path $(TOOLKIT_ROOT)/Cargo.toml -p mnemonic-toolkit --bin mnemonic --
MD_BIN       ?= cargo run --quiet --manifest-path $(WORKSPACE_ROOT)/descriptor-mnemonic/Cargo.toml -p md-cli --bin md --
MS_BIN       ?= cargo run --quiet --manifest-path $(WORKSPACE_ROOT)/mnemonic-secret/Cargo.toml -p ms-cli --bin ms --
MK_BIN       ?= cargo run --quiet --manifest-path $(WORKSPACE_ROOT)/mnemonic-key/Cargo.toml -p mk-cli --bin mk --

# Cross-format-recipes fixture corpus (consumed by verify-examples.sh).
FIXTURES_DIR ?= $(TOOLKIT_ROOT)/crates/mnemonic-toolkit/tests/fixtures/wallet_import

# ---------------------------------------------------------------------------
# Tools (override with PANDOC=/path/to/pandoc etc.)
# ---------------------------------------------------------------------------
PANDOC      ?= pandoc
XELATEX     ?= xelatex
MAKEINDEX   ?= makeindex
DOCKER      ?= docker
DOCKER_IMAGE ?= mnemonic-manual-build:latest

# ---------------------------------------------------------------------------
# Filter / build mode
# ---------------------------------------------------------------------------
MERMAID_FILTER ?= skip

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
PDF_FILTER_ARGS := --lua-filter $(FILTERS_DIR)/primer-box.lua --lua-filter $(FILTERS_DIR)/wrap-long-code.lua

ifeq ($(MERMAID_FILTER),on)
	PDF_FILTER_ARGS += --filter mermaid-filter
else ifeq ($(MERMAID_FILTER),skip)
	PDF_FILTER_ARGS += --lua-filter $(FILTERS_DIR)/mermaid-cache-filter.lua
	# Absolute path: pandoc may be invoked from any cwd; the Lua filter
	# resolves cache paths via this env var, so a relative value would
	# silently miss-resolve.
	export MERMAID_CACHE_DIR := $(abspath $(SRC_DIR)/../figures/cache)
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
md: $(BUILD_DIR)/m-format-manual.md

$(BUILD_DIR)/m-format-manual.md: $(MD_SRC) $(FILTERS_DIR)/strip-latex-from-md.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
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
# PDF render path: pandoc --to latex → xelatex → makeindex → xelatex → xelatex
# ---------------------------------------------------------------------------
.PHONY: pdf
pdf: $(BUILD_DIR)/m-format-manual.pdf

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

$(BUILD_DIR)/m-format-manual.tex: $(PDF_SRC) $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml $(PANDOC_DIR)/preamble.tex $(TEMPLATES_DIR)/manual.latex | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to latex \
		--standalone \
		--toc \
		--toc-depth=3 \
		--top-level-division=chapter \
		--template=$(TEMPLATES_DIR)/manual.latex \
		-H $(PANDOC_DIR)/preamble.tex \
		$(PDF_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--output $@ \
		$(PDF_SRC)

$(BUILD_DIR)/m-format-manual.pdf: $(BUILD_DIR)/m-format-manual.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-manual.tex
	# Run makeindex only if the .idx file was produced (xelatex emits it
	# only when the source contains at least one \index{} marker).
	cd $(BUILD_DIR) && if [ -f m-format-manual.idx ]; then \
	    $(MAKEINDEX) m-format-manual.idx; \
	else \
	    echo '[pdf] no \\index{} markers in source; skipping makeindex'; \
	fi
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-manual.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-manual.tex

# ---------------------------------------------------------------------------
# Dockerized PDF build (canonical)
# ---------------------------------------------------------------------------
.PHONY: pdf-docker docker-image
docker-image:
	$(DOCKER) build -t $(DOCKER_IMAGE) -f $(MANUAL_DIR)/Dockerfile.build $(MANUAL_DIR)

pdf-docker: docker-image
	$(DOCKER) run --rm \
		-u $$(id -u):$$(id -g) \
		-v $(TOOLKIT_ROOT):/work \
		-w /work/docs/manual \
		$(DOCKER_IMAGE) \
		make pdf MERMAID_FILTER=$(MERMAID_FILTER)

# ---------------------------------------------------------------------------
# Pre-rendered mermaid figures cache (offline build fallback)
# ---------------------------------------------------------------------------
.PHONY: figures-cache figures-cache-verify
figures-cache:
	python3 $(TOOLKIT_ROOT)/docs/tools/render-mermaid-cache.py \
		--doc-root $(MANUAL_DIR) \
		--src-glob 'src/**/*.md' \
		--cache-dir figures/cache

figures-cache-verify:
	@python3 $(TOOLKIT_ROOT)/docs/tools/render-mermaid-cache.py \
		--doc-root $(MANUAL_DIR) \
		--src-glob 'src/**/*.md' \
		--cache-dir figures/cache \
		--verify

# Run the cache-verify gate as a lint prerequisite (Make merges multiple `lint:` rules).
lint: figures-cache-verify

# ---------------------------------------------------------------------------
# Worked-example regeneration / verification
# ---------------------------------------------------------------------------
.PHONY: verify-examples
verify-examples:
	@bash $(TESTS_DIR)/verify-examples.sh \
		MNEMONIC_BIN="$(MNEMONIC_BIN)" \
		MD_BIN="$(MD_BIN)" \
		MS_BIN="$(MS_BIN)" \
		MK_BIN="$(MK_BIN)" \
		FIXTURES_DIR="$(FIXTURES_DIR)" \
		TRANSCRIPTS="$(TRANSCRIPTS)"

# `make audit` umbrella target: lint (static gates) + verify-examples
# (transcript replay against real binaries). CI runs `make audit`; local
# fast iteration uses `make lint` alone.
.PHONY: audit
audit: lint verify-examples

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
		MS_BIN="$(MS_BIN)" \
		MK_BIN="$(MK_BIN)"

# ---------------------------------------------------------------------------
# Filter smoke test (Phase 0 verification)
# ---------------------------------------------------------------------------
.PHONY: filter-smoke
filter-smoke:
	@bash $(TESTS_DIR)/filter-smoke.sh \
		MANUAL_DIR="$(MANUAL_DIR)" \
		PANDOC="$(PANDOC)" \
		XELATEX="$(XELATEX)" \
		MAKEINDEX="$(MAKEINDEX)"

# ---------------------------------------------------------------------------
# Release-asset attach (called from CI or by hand on a tagged build)
# ---------------------------------------------------------------------------
.PHONY: release-attach
release-attach: pdf-docker
ifndef VERSION
	$(error VERSION must be set, e.g. `make release-attach VERSION=manual-v0.1.0`)
endif
	gh release upload $(VERSION) \
		$(BUILD_DIR)/m-format-manual.pdf \
		--clobber

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(SRC_DIR)/99-build-banner.md
