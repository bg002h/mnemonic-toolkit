//! v0.27.0 presence-smoke tests for canonical design-tree artifacts.
//!
//! These tests assert that load-bearing design records exist at their
//! documented locations. They're cheap regression guards against accidental
//! deletion or renaming during future cycles (a moved file forces a
//! conscious decision to update the test + CLAUDE.md cross-cite together).
//!
//! Closes `coordinator-runbook-into-design-dir` FOLLOWUP (v0.27.0 Phase 1).

use std::path::PathBuf;

fn design_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../design")
}

#[test]
fn three_way_merge_runbook_lives_in_design_dir() {
    let path = design_root().join("PLAN_v0_26_0_three_way_merge.md");
    assert!(
        path.exists(),
        "expected canonical multi-instance coordination playbook at {} (per CLAUDE.md Conventions cross-cite + coordinator-runbook-into-design-dir FOLLOWUP)",
        path.display()
    );
}
