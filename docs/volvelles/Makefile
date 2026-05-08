# mnemonic-toolkit/docs/volvelles/Makefile
#
# Build the m-format constellation paper-computer wheels (mk1-regular,
# mk1-long, md1-regular).
#
# Targets:
#   make verify         — run the drift gate (codec round-trip ↔ Python helper)
#   make pdf            — render the three wheel PDFs to build/ (depends on verify)
#   make release-attach — copy build/*.pdf to docs/volvelles/*.pdf (release-source path)
#   make clean          — remove build/, generated bch-params.toml + *.cells.tex, LaTeX byproducts
#   make help           — print this help
#
# `verify` is the default target: cheap, fast, and the gate every other
# target depends on (directly or transitively).
#
# Variables (override on the command line):
#   PYTHON   — Python interpreter (default python3; needs ≥ 3.11 for tomllib)
#   CARGO    — cargo binary (default cargo)
#   XELATEX  — XeLaTeX binary (default xelatex)

# ---------------------------------------------------------------------------
# Repo-relative paths
# ---------------------------------------------------------------------------
VOLVELLES_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TOOLS_DIR     := $(abspath $(VOLVELLES_DIR)/../tools)
VERIFY_DIR    := $(VOLVELLES_DIR)/verify
BUILD_DIR     := $(VOLVELLES_DIR)/build

PARAMS_TOML   := $(VOLVELLES_DIR)/bch-params.toml
HELPER        := $(TOOLS_DIR)/make-volvelle-data.py

WHEELS        := mk-regular mk-long md-regular
TEX_SRCS      := $(addprefix $(VOLVELLES_DIR)/,$(addsuffix .tex,$(WHEELS)))
CELLS_TEX     := $(addprefix $(VOLVELLES_DIR)/,$(addsuffix .cells.tex,$(WHEELS)))
PDFS          := $(addprefix $(BUILD_DIR)/,$(addsuffix .pdf,$(WHEELS)))
RELEASE_PDFS  := $(addprefix $(VOLVELLES_DIR)/,$(addsuffix .pdf,$(WHEELS)))

# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------
PYTHON  ?= python3
CARGO   ?= cargo
XELATEX ?= xelatex

# ---------------------------------------------------------------------------
# Default target
# ---------------------------------------------------------------------------
.DEFAULT_GOAL := all

.PHONY: all
all: verify

.PHONY: help
help:
	@grep -E '^# ?(make |  )' $(MAKEFILE_LIST) | sed 's/^# \?//'

# ---------------------------------------------------------------------------
# Drift gate
# ---------------------------------------------------------------------------
# Regenerate bch-params.toml whenever the verify crate's lockfile, manifest,
# or sources change. The recipe `cd`s into verify/ so the standalone-workspace
# crate resolves correctly (it is not a member of the toolkit's main workspace).
$(PARAMS_TOML): $(VERIFY_DIR)/Cargo.toml $(VERIFY_DIR)/Cargo.lock $(VERIFY_DIR)/src/main.rs
	cd $(VERIFY_DIR) && $(CARGO) run --quiet --bin print-constants > $(PARAMS_TOML)

# Cargo.lock is gitignored (per docs/volvelles/.gitignore); on a clean clone
# it doesn't exist until cargo generates it. This rule ensures `make verify`
# works on a fresh checkout by invoking `cargo generate-lockfile` if needed.
$(VERIFY_DIR)/Cargo.lock: $(VERIFY_DIR)/Cargo.toml
	cd $(VERIFY_DIR) && $(CARGO) generate-lockfile

.PHONY: verify
verify: $(PARAMS_TOML)
	$(PYTHON) $(HELPER) --params $(PARAMS_TOML) --check-only

# ---------------------------------------------------------------------------
# PDF render
# ---------------------------------------------------------------------------
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Emit per-format .cells.tex fragments (writes into VOLVELLES_DIR by default).
# Re-runs the drift check internally before emitting; we still depend on
# `verify` so a failing gate halts before this recipe fires. Grouped target
# (`&:`, GNU make ≥ 4.3) — one recipe invocation produces all three files.
$(CELLS_TEX) &: $(PARAMS_TOML) $(HELPER)
	$(PYTHON) $(HELPER) --params $(PARAMS_TOML) --out-dir $(VOLVELLES_DIR)

# Pattern rule: build/<wheel>.pdf from <wheel>.tex + <wheel>.cells.tex.
# `cd`s into BUILD_DIR so xelatex's auxiliary files (.aux, .log, .toc) land
# there rather than polluting the source directory; -output-directory is
# avoided because TeX's \input{} resolution is simpler with cwd-relative paths.
# Order-only `verify` prereq ensures the drift gate runs before xelatex even
# under parallel make (-j).
$(BUILD_DIR)/%.pdf: $(VOLVELLES_DIR)/%.tex $(VOLVELLES_DIR)/%.cells.tex | verify $(BUILD_DIR)
	cd $(BUILD_DIR) && TEXINPUTS="$(VOLVELLES_DIR):$$TEXINPUTS" \
		$(XELATEX) -interaction=nonstopmode -halt-on-error \
		-jobname=$* $(VOLVELLES_DIR)/$*.tex

.PHONY: pdf
# If a wheel `.tex` source is missing (e.g. Task 3 hasn't shipped yet), make's
# standard pattern-rule resolution emits a clear "No rule to make target
# '<wheel>.tex'" error — graceful failure with no extra plumbing required.
pdf: $(PDFS)

# ---------------------------------------------------------------------------
# Release-attach: copy rendered PDFs to the tracked release-source path.
# ---------------------------------------------------------------------------
.PHONY: release-attach
release-attach: pdf
	@for w in $(WHEELS); do \
		cp $(BUILD_DIR)/$$w.pdf $(VOLVELLES_DIR)/$$w.pdf; \
		echo "release-attach: $(VOLVELLES_DIR)/$$w.pdf"; \
	done

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(PARAMS_TOML)
	rm -f $(CELLS_TEX)
	rm -f $(VOLVELLES_DIR)/*.aux $(VOLVELLES_DIR)/*.log $(VOLVELLES_DIR)/*.toc
