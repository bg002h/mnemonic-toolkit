# mnemonic-toolkit/docs/manual-gui/Makefile
#
# Build the mnemonic-gui user manual to markdown, PDF, and HTML.
#
# Targets:
#   make md            — concatenated markdown to build/m-format-gui-manual.md
#   make pdf           — typeset PDF to build/m-format-gui-manual.pdf (needs pandoc + xelatex; consumes figures/cache/)
#   make html          — single-file HTML to build/m-format-gui-manual.html (for GitHub Pages deploy)
#   make pdf-docker    — same as `make pdf`, inside Dockerfile.build (pinned, reproducible)
#   make figures-cache — pre-render mermaid blocks to figures/cache/ (rare; needs mmdc + chromium locally)
#   make verify-examples — re-run worked-example transcripts against locally-built CLIs
#   make verify-examples-gui — regenerate the 61 GUI form renders with the pinned gui-render and diff == committed (census + secret-hygiene)
#   make gui-example-md    — gui_example tutorial book to build/gui_example.md
#   make gui-example-html  — gui_example tutorial book to build/gui_example.html (embeds the tutorial figures as data-URIs)
#   make gui-example-pdf   — gui_example tutorial book to build/gui_example.pdf (RELEASE-ATTACH ONLY; never committed)
#   make lint          — markdownlint + cspell + lychee + gui-schema-coverage + outline-coverage + glossary-coverage + index + gui-form-xref + verify-figures-gui + verify-tutorial-figures + verify-tutorial-transcripts + tutorial-xref
#   make clean         — remove build/ and LaTeX byproducts
#   make help          — print this help
#
# Variables (override on the command line or in CI):
#   MNEMONIC_BIN, MD_BIN, MS_BIN, MK_BIN — CLI invocation strings used by verify-examples and flag-coverage.
#   MERMAID_FILTER  — 'skip' (default) consumes figures/cache/ snapshots; 'on' runs mermaid-filter
#                      (needs chromium + mermaid-filter locally — rare, only for cache regen review).
#   PANDOC, XELATEX, MAKEINDEX — tool paths (override if not on $PATH).
#   DOCKER_IMAGE — name to tag the build image as (default mnemonic-gui-manual-build:latest).

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

# gui_example tutorial book (a sibling book inside manual-gui): its
# chapter sources, the byte-copied figure/transcript corpora, and the
# lexicographically-ordered source list the gui-example-* targets build.
TUTORIAL_DIR         := $(MANUAL_DIR)/tutorial
FIGURES_TUTORIAL     := $(FIGURES_DIR)/tutorial
TRANSCRIPTS_TUTORIAL := $(TRANSCRIPTS)/tutorial
GUI_EXAMPLE_SRC      := $(shell find $(TUTORIAL_DIR) -type f -name '*.md' 2>/dev/null | LC_ALL=C sort)
GUI_EXAMPLE_TITLE    ?= The mnemonic-gui Tutorial — Worked Journeys
GUI_EXAMPLE_SUBTITLE ?= Five end-to-end journeys through the mnemonic-gui desktop application

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
DOCKER_IMAGE ?= mnemonic-gui-manual-build:latest

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
#
# include-transcript.lua is PREPENDED (first) on both chains so a fenced block's
# `include=` attribute is resolved into the captured transcript BEFORE any other
# transform sees it. It resolves the transcript via TRANSCRIPTS_DIR (exported
# below). Carried forward from the manual / quickstart Makefiles for P4's output
# blocks; the manual-gui transcripts dir is .gitkeep-only until P4 fills it.
# See pandoc/filters/include-transcript.lua + the cycle-C P0a spec.
MD_FILTER_ARGS  := --lua-filter $(FILTERS_DIR)/include-transcript.lua --lua-filter $(FILTERS_DIR)/strip-latex-from-md.lua --lua-filter $(FILTERS_DIR)/primer-box.lua
PDF_FILTER_ARGS := --lua-filter $(FILTERS_DIR)/include-transcript.lua --lua-filter $(FILTERS_DIR)/primer-box.lua --lua-filter $(FILTERS_DIR)/wrap-long-code.lua

# Absolute path: pandoc may be invoked from any cwd; include-transcript.lua
# resolves the transcript file via this env var, so a relative value would
# silently miss-resolve.
export TRANSCRIPTS_DIR := $(abspath $(TRANSCRIPTS))

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
md: $(BUILD_DIR)/m-format-gui-manual.md

$(BUILD_DIR)/m-format-gui-manual.md: $(MD_SRC) $(FILTERS_DIR)/include-transcript.lua $(FILTERS_DIR)/strip-latex-from-md.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
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
pdf: $(BUILD_DIR)/m-format-gui-manual.pdf

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

$(BUILD_DIR)/m-format-gui-manual.tex: $(PDF_SRC) $(FILTERS_DIR)/include-transcript.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml $(PANDOC_DIR)/preamble.tex $(TEMPLATES_DIR)/manual.latex | $(BUILD_DIR)
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

$(BUILD_DIR)/m-format-gui-manual.pdf: $(BUILD_DIR)/m-format-gui-manual.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-gui-manual.tex
	# Run makeindex only if the .idx file was produced (xelatex emits it
	# only when the source contains at least one \index{} marker).
	cd $(BUILD_DIR) && if [ -f m-format-gui-manual.idx ]; then \
	    $(MAKEINDEX) m-format-gui-manual.idx; \
	else \
	    echo '[pdf] no \\index{} markers in source; skipping makeindex'; \
	fi
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-gui-manual.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error m-format-gui-manual.tex

# ---------------------------------------------------------------------------
# HTML render path: --to html5 (single-file with embedded CSS for gh-pages)
# ---------------------------------------------------------------------------
.PHONY: html
html: $(BUILD_DIR)/m-format-gui-manual.html

# --resource-path: the GUI-Forms gallery chapters reference their
# screenshots FILE-relative (`../../figures/gui/<stem>.png` from
# src/75-gui-forms/) so lychee's offline link check resolves them.
# pandoc, however, resolves image resources against the CWD/resource
# path — not the source file — so `--self-contained` embedding needs
# `$(SRC_DIR)/75-gui-forms` on the resource path for the file-relative
# form to resolve (`src/75-gui-forms/../../figures/gui/…` == the
# committed figure). NOTE: pandoc exits 0 with a WARNING on a missing
# image — a green `make html` proves nothing about embeds; the gate is
# the positive embed census (grep -c 'img src="data:' == 61).
$(BUILD_DIR)/m-format-gui-manual.html: $(PDF_SRC) $(FILTERS_DIR)/include-transcript.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to html5 \
		--standalone \
		--self-contained \
		--resource-path=.:$(SRC_DIR)/75-gui-forms \
		--toc \
		--toc-depth=3 \
		--top-level-division=chapter \
		$(PDF_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--output $@ \
		$(PDF_SRC)

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
		-w /work/docs/manual-gui \
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
# Generated GUI form renders — fidelity + census gate (SEPARATE from the
# symlinked verify-examples.sh above).
# ---------------------------------------------------------------------------
# The committed structural form renders the manual embeds via
# `include="gui/<tab>-<sub>.gui"`. The gate regenerates them with the PINNED
# `gui-render` (headless, egui-free `--no-default-features`) and byte-diffs
# == the committed transcripts/gui/ tree (fail-closed), plus a census that
# every subcommand in the pinned GUI schema has a committed render.
#
# GUI_RENDER_BIN default builds + runs gui-render straight from the pinned
# mnemonic-gui checkout ($(MANUAL_GUI_UPSTREAM_ROOT), the same source the
# lint's schema-coverage reads). `?=` so CI overrides it to an installed
# pinned binary (`cargo install --git … --tag … --no-default-features
# --bin gui-render`, manual-gui.yml verify-examples-gui job).
#
# `env RUSTUP_TOOLCHAIN=$(GUI_RENDER_TOOLCHAIN)`: this repo's root
# rust-toolchain.toml pins rustc 1.85 (the toolkit MSRV), but mnemonic-gui's
# MSRV is higher (1.88 at v0.53.0). rustup resolves rust-toolchain.toml from
# the build cwd (inside this repo) upward, so an un-overridden `cargo run`
# would pick the 1.85 pin and fail the GUI build. RUSTUP_TOOLCHAIN has top
# priority and overrides the file pin; `stable` on any reasonably current
# box is ≥ the GUI MSRV. CI overrides GUI_RENDER_BIN wholesale, so this only
# affects the local default. EXPECTED_GUI_RENDER_COUNT is the pinned GUI's
# subcommand total (census key; bumps in lockstep with the GUI pin).
TRANSCRIPTS_GUI            := $(TRANSCRIPTS)/gui
EXPECTED_GUI_RENDER_COUNT ?= 61
GUI_RENDER_TOOLCHAIN      ?= stable
GUI_RENDER_BIN            ?= env RUSTUP_TOOLCHAIN=$(GUI_RENDER_TOOLCHAIN) cargo run --quiet --manifest-path $(MANUAL_GUI_UPSTREAM_ROOT)/Cargo.toml --no-default-features --bin gui-render --

.PHONY: verify-examples-gui
verify-examples-gui:
	@bash $(TESTS_DIR)/verify-examples-gui.sh \
		GUI_RENDER_BIN="$(GUI_RENDER_BIN)" \
		TRANSCRIPTS_GUI="$(TRANSCRIPTS_GUI)" \
		EXPECTED_COUNT="$(EXPECTED_GUI_RENDER_COUNT)"

# ---------------------------------------------------------------------------
# gui_example tutorial book — its own md / html / pdf targets
# ---------------------------------------------------------------------------
# The tutorial book (chapters under $(TUTORIAL_DIR), figures under
# $(FIGURES_TUTORIAL), transcripts under $(TRANSCRIPTS_TUTORIAL)) builds to
# build/gui_example.{md,html,pdf} with the SAME pandoc→xelatex recipe as the
# reference manual but its own title page. It reuses include-transcript.lua
# (transcript fenced includes resolve via the exported TRANSCRIPTS_DIR, so
# `include="tutorial/<name>"` -> transcripts/tutorial/<name>) and the shared
# preamble/template. Figures are referenced FILE-relative from tutorial/*.md
# as `../figures/tutorial/<name>.png`: (i) lychee --offline resolves them
# file-relative; (ii) HTML embedding needs $(TUTORIAL_DIR) on the resource
# path (pandoc resolves image resources against the resource path, not the
# source file); (iii) the PDF resolves them at xelatex time (build/ is one
# level under manual-gui, so `../figures/tutorial/…` from build/ is the
# committed figure). Per the release-attach-only decision (SPEC §3.2c
# amended 2026-07-05) the PDF is built+gated here and in CI but is NOT
# committed; the manual-gui release job attaches it on `manual-gui-v*` tags.
.PHONY: gui-example-md gui-example-html gui-example-pdf

gui-example-md: $(BUILD_DIR)/gui_example.md
$(BUILD_DIR)/gui_example.md: $(GUI_EXAMPLE_SRC) $(FILTERS_DIR)/include-transcript.lua $(FILTERS_DIR)/strip-latex-from-md.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to gfm \
		--standalone \
		--toc \
		--toc-depth=2 \
		$(MD_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--metadata title="$(GUI_EXAMPLE_TITLE)" \
		--metadata subtitle="$(GUI_EXAMPLE_SUBTITLE)" \
		--output $@ \
		$(GUI_EXAMPLE_SRC)

# --resource-path: figures are referenced `../figures/tutorial/<name>.png`
# from tutorial/*.md; pandoc resolves image resources against the resource
# path (not the source file), so $(TUTORIAL_DIR) must be on it for the
# --self-contained data-URI embed to resolve. NOTE: pandoc exits 0 with a
# WARNING on a missing image — the gate is the POSITIVE embed census in
# manual-gui.yml (grep -c 'src="data:image/png' == the committed
# figures/tutorial PNG count), not a green build.
gui-example-html: $(BUILD_DIR)/gui_example.html
$(BUILD_DIR)/gui_example.html: $(GUI_EXAMPLE_SRC) $(FILTERS_DIR)/include-transcript.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to html5 \
		--standalone \
		--self-contained \
		--resource-path=.:$(TUTORIAL_DIR) \
		--toc \
		--toc-depth=2 \
		--top-level-division=chapter \
		$(PDF_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--metadata title="$(GUI_EXAMPLE_TITLE)" \
		--metadata subtitle="$(GUI_EXAMPLE_SUBTITLE)" \
		--output $@ \
		$(GUI_EXAMPLE_SRC)

gui-example-pdf: $(BUILD_DIR)/gui_example.pdf

$(BUILD_DIR)/gui_example.tex: $(GUI_EXAMPLE_SRC) $(FILTERS_DIR)/include-transcript.lua $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml $(PANDOC_DIR)/preamble.tex $(TEMPLATES_DIR)/manual.latex | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to latex \
		--standalone \
		--toc \
		--toc-depth=2 \
		--top-level-division=chapter \
		--template=$(TEMPLATES_DIR)/manual.latex \
		-H $(PANDOC_DIR)/preamble.tex \
		$(PDF_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--metadata title="$(GUI_EXAMPLE_TITLE)" \
		--metadata subtitle="$(GUI_EXAMPLE_SUBTITLE)" \
		--output $@ \
		$(GUI_EXAMPLE_SRC)

$(BUILD_DIR)/gui_example.pdf: $(BUILD_DIR)/gui_example.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error gui_example.tex
	cd $(BUILD_DIR) && if [ -f gui_example.idx ]; then \
	    $(MAKEINDEX) gui_example.idx; \
	else \
	    echo '[pdf] no \\index{} markers in gui_example; skipping makeindex'; \
	fi
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error gui_example.tex
	cd $(BUILD_DIR) && $(XELATEX) -interaction=nonstopmode -halt-on-error gui_example.tex

# ---------------------------------------------------------------------------
# Lint
# ---------------------------------------------------------------------------
# Sibling-checkout default for the mnemonic-gui source the lint reads.
# Override via env var MANUAL_GUI_UPSTREAM_ROOT for CI / out-of-tree builds.
MANUAL_GUI_UPSTREAM_ROOT ?= $(abspath $(WORKSPACE_ROOT)/mnemonic-gui)

# FIGURES_GUI mirrors the TRANSCRIPTS_GUI threading pattern above: the
# committed GUI-form screenshot corpus (figures/gui/<tab>-<sub>.png),
# byte-compared by lint phase 9 (verify-figures-gui) against the pinned
# mnemonic-gui checkout's tests/snapshots/forms/ — census
# $(EXPECTED_GUI_RENDER_COUNT) both directions, fail-closed.
FIGURES_GUI := $(FIGURES_DIR)/gui

.PHONY: lint
lint:
	@bash $(TESTS_DIR)/lint.sh \
		SRC_DIR="$(SRC_DIR)" \
		BUILD_DIR="$(BUILD_DIR)" \
		TESTS_DIR="$(TESTS_DIR)" \
		MANUAL_GUI_UPSTREAM_ROOT="$(MANUAL_GUI_UPSTREAM_ROOT)" \
		TRANSCRIPTS_GUI="$(TRANSCRIPTS_GUI)" \
		FIGURES_GUI="$(FIGURES_GUI)" \
		EXPECTED_GUI_RENDER_COUNT="$(EXPECTED_GUI_RENDER_COUNT)" \
		FIGURES_TUTORIAL="$(FIGURES_TUTORIAL)" \
		TRANSCRIPTS_TUTORIAL="$(TRANSCRIPTS_TUTORIAL)" \
		TUTORIAL_DIR="$(TUTORIAL_DIR)" \
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
	$(error VERSION must be set, e.g. `make release-attach VERSION=manual-gui-v1.0.0`)
endif
	gh release upload $(VERSION) \
		$(BUILD_DIR)/m-format-gui-manual.pdf \
		$(BUILD_DIR)/m-format-gui-manual.html \
		--clobber

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(SRC_DIR)/99-build-banner.md
