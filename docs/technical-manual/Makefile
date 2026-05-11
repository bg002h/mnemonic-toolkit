# mnemonic-toolkit/docs/technical-manual/Makefile
#
# Build the m-format constellation TECHNICAL manual to markdown and PDF.
# Companion to docs/manual/ (the end-user manual); same pandoc + xelatex
# pipeline, separate source tree, separate output filename.
#
# Targets:
#   make md            — concatenated markdown to build/m-format-technical-manual.md
#   make pdf           — typeset PDF to build/m-format-technical-manual.pdf (needs pandoc + xelatex; consumes figures/cache/)
#   make pdf-docker    — same as `make pdf`, inside Dockerfile.build (pinned, reproducible)
#   make figures-cache — pre-render mermaid blocks to figures/cache/ (rare; needs mmdc + chromium locally)
#   make verify-examples — re-run worked-example transcripts against locally-built CLIs
#   make lint          — markdownlint + cspell + lychee + api-surface-coverage (hint, warning-only) + glossary-coverage + index bidirectional
#   make clean         — remove build/ and LaTeX byproducts
#   make help          — print this help
#
# Variables (override on the command line or in CI):
#   MNEMONIC_BIN, MD_BIN, MS_BIN, MK_BIN — CLI invocation strings used by verify-examples and the Phase-4.5 api-surface-coverage helper.
#   MERMAID_FILTER  — 'skip' (default) consumes figures/cache/ snapshots; 'on' runs mermaid-filter
#                      (needs chromium + mermaid-filter locally — rare, only for cache regen review).
#   PANDOC, XELATEX, MAKEINDEX — tool paths (override if not on $PATH).
#   DOCKER_IMAGE — name to tag the build image as (default mnemonic-tech-manual-build:latest).
#   SOURCE_DATE_EPOCH — Unix timestamp; when set, the build banner timestamp,
#                      PDF /CreationDate, and PDF /ModDate all derive from it,
#                      producing byte-identical PDFs across repeated builds
#                      (SPEC §7 A11 same-host reproducibility). Pairs with
#                      FORCE_SOURCE_DATE=1 (auto-exported to xelatex).

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

# ---------------------------------------------------------------------------
# Tools (override with PANDOC=/path/to/pandoc etc.)
# ---------------------------------------------------------------------------
PANDOC      ?= pandoc
XELATEX     ?= xelatex
MAKEINDEX   ?= makeindex
DOCKER      ?= docker
DOCKER_IMAGE ?= mnemonic-tech-manual-build:latest

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
PDF_FILTER_ARGS := --lua-filter $(FILTERS_DIR)/primer-box.lua

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
md: $(BUILD_DIR)/m-format-technical-manual.md

$(BUILD_DIR)/m-format-technical-manual.md: $(MD_SRC) $(FILTERS_DIR)/strip-latex-from-md.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
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
pdf: $(BUILD_DIR)/m-format-technical-manual.pdf

# Regenerate the build banner each PDF build so $(GIT_SHA) / $(BUILD_DATE)
# are fresh. SOURCE_DATE_EPOCH (set externally) overrides BUILD_DATE for
# byte-identical reproducible builds; combined with FORCE_SOURCE_DATE=1
# exported to xelatex, this freezes PDF /CreationDate + /ModDate and the
# banner timestamp together. SPEC §7 A11 same-host byte-identity gate
# uses this mechanism.
GIT_SHA    ?= $(shell git -C $(TOOLKIT_ROOT) rev-parse --short HEAD 2>/dev/null || echo unknown)
ifdef SOURCE_DATE_EPOCH
BUILD_DATE ?= $(shell date -u -d @$(SOURCE_DATE_EPOCH) +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -r $(SOURCE_DATE_EPOCH) +%Y-%m-%dT%H:%M:%SZ)
export SOURCE_DATE_EPOCH
export FORCE_SOURCE_DATE := 1
else
BUILD_DATE ?= $(shell date -u +%Y-%m-%dT%H:%M:%SZ)
endif

$(SRC_DIR)/99-build-banner.md: FORCE
	@printf '%s\n' \
		'# Build banner' \
		'' \
		'Built from commit `$(GIT_SHA)` on `$(BUILD_DATE)`.' \
		'' \
		'_This page is included in the PDF render only._' > $@

.PHONY: FORCE
FORCE:

$(BUILD_DIR)/m-format-technical-manual.tex: $(PDF_SRC) $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml $(PANDOC_DIR)/preamble.tex $(TEMPLATES_DIR)/manual.latex | $(BUILD_DIR)
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

$(BUILD_DIR)/m-format-technical-manual.pdf: $(BUILD_DIR)/m-format-technical-manual.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-technical-manual.tex
	# Run makeindex only if the .idx file was produced (xelatex emits it
	# only when the source contains at least one \index{} marker).
	cd $(BUILD_DIR) && if [ -f m-format-technical-manual.idx ]; then \
	    $(MAKEINDEX) m-format-technical-manual.idx; \
	else \
	    echo '[pdf] no \\index{} markers in source; skipping makeindex'; \
	fi
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-technical-manual.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-technical-manual.tex

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
		-w /work/docs/technical-manual \
		$(if $(SOURCE_DATE_EPOCH),-e SOURCE_DATE_EPOCH=$(SOURCE_DATE_EPOCH) -e FORCE_SOURCE_DATE=1,) \
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
	$(error VERSION must be set, e.g. `make release-attach VERSION=tech-manual-v0.1.0`)
endif
	gh release upload $(VERSION) \
		$(BUILD_DIR)/m-format-technical-manual.pdf \
		--clobber

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(SRC_DIR)/99-build-banner.md
