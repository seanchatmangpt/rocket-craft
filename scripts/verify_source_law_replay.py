#!/usr/bin/env python3
"""R2a SOURCE-LAW REPLAY GATE (read-only proof).

Proves that ontology/all_merged.ttl is a lawful, contamination-free replay of
ontology/source_law/*.ttl. Writes ONLY:
  - SOURCE_LAW_REPLAY_REPORT.json
  - SOURCE_LAW_REPLAY_REPORT.md
at repo root.

Contamination check = ALL of:
  (A) HEAD all_merged.ttl carries a `# --- Source: <name> ---` banner for
      every source_law/*.ttl, each exactly once.
  (B) A fresh merge (delete + scripts/merge_ontology.py) is byte-identical to
      HEAD all_merged.ttl.
  (C) No orphan content: every banner in the fresh merge maps to a real source
      file and every source file maps to a banner (bijection).

The KNOWN failure mode (HEAD hand-merged, no banners) makes (A)+(B) fail ->
PARTIAL_ALIVE, next_action = R2b triage. This script never edits source.
"""
import hashlib
import os
import re
import subprocess
import sys
import json
import datetime

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
MERGED = os.path.join(REPO, "ontology", "all_merged.ttl")
SRC_DIR = os.path.join(REPO, "ontology", "source_law")
MERGE_SCRIPT = os.path.join(REPO, "scripts", "merge_ontology.py")
BANNER_RE = re.compile(r"^# --- Source: (.+?) ---\s*$", re.MULTILINE)


def b3(data: bytes) -> str:
    try:
        from hashlib import blake2b  # fallback if blake3 unavailable
    except Exception:
        pass
    # Prefer real BLAKE3 via b3sum for receipt-law parity.
    p = subprocess.run(["b3sum", "--no-names"], input=data,
                       capture_output=True)
    if p.returncode == 0:
        return p.stdout.decode().strip()
    return "sha256:" + hashlib.sha256(data).hexdigest()


def b3_file(path: str) -> str:
    with open(path, "rb") as f:
        return b3(f.read())


def main():
    report = {
        "gate": "R2a_source_law_replay",
        "timestamp_utc": datetime.datetime.utcnow().isoformat() + "Z",
        "repo": REPO,
        "checks": {},
        "source_law_receipts": {},
    }

    # --- snapshot working-tree merged b3 ---
    wt_b3 = b3_file(MERGED) if os.path.exists(MERGED) else None
    report["working_tree_merged_b3"] = wt_b3

    # --- capture HEAD all_merged.ttl ---
    head = subprocess.run(
        ["git", "show", "HEAD:ontology/all_merged.ttl"],
        cwd=REPO, capture_output=True)
    if head.returncode != 0:
        report["checks"]["head_available"] = False
        report["fatal"] = "HEAD:ontology/all_merged.ttl unavailable: " + head.stderr.decode()
        _emit(report)
        return 2
    head_bytes = head.stdout
    head_b3 = b3(head_bytes)
    report["head_merged_b3"] = head_b3
    head_banners = BANNER_RE.findall(head_bytes.decode("utf-8", "replace"))
    report["head_banner_count"] = len(head_banners)

    # --- source_law inventory + receipts (R2 receipt leaves) ---
    src_files = sorted(
        f for f in os.listdir(SRC_DIR) if f.endswith(".ttl"))
    for f in src_files:
        report["source_law_receipts"][f] = b3_file(os.path.join(SRC_DIR, f))
    report["source_law_count"] = len(src_files)

    # --- fresh merge: delete + regenerate (proof of replay) ---
    # Preserve working tree exactly by restoring afterwards.
    pre_merge_bytes = None
    if os.path.exists(MERGED):
        with open(MERGED, "rb") as fh:
            pre_merge_bytes = fh.read()
    try:
        if os.path.exists(MERGED):
            os.remove(MERGED)
        merged_run = subprocess.run(
            [sys.executable, MERGE_SCRIPT], cwd=REPO, capture_output=True)
        report["checks"]["merge_script_ok"] = merged_run.returncode == 0
        if merged_run.returncode != 0:
            report["fatal"] = "merge_ontology.py failed: " + merged_run.stderr.decode()
        with open(MERGED, "rb") as fh:
            fresh_bytes = fh.read()
    finally:
        # restore the working-tree file we snapshotted (read-only contract)
        if pre_merge_bytes is not None:
            with open(MERGED, "wb") as fh:
                fh.write(pre_merge_bytes)

    fresh_b3 = b3(fresh_bytes)
    report["fresh_merge_b3"] = fresh_b3
    fresh_banners = BANNER_RE.findall(fresh_bytes.decode("utf-8", "replace"))
    report["fresh_banner_count"] = len(fresh_banners)

    # (A) HEAD has a banner per source file, each exactly once
    head_counts = {}
    for b in head_banners:
        head_counts[b] = head_counts.get(b, 0) + 1
    a_ok = (set(head_counts) == set(src_files) and
            all(v == 1 for v in head_counts.values()))
    report["checks"]["A_head_banner_bijection"] = a_ok

    # (B) fresh merge byte-identical to HEAD
    b_ok = fresh_b3 == head_b3
    report["checks"]["B_fresh_equals_head"] = b_ok

    # (C) fresh-merge banner bijection (no orphan content)
    fresh_counts = {}
    for b in fresh_banners:
        fresh_counts[b] = fresh_counts.get(b, 0) + 1
    c_ok = (set(fresh_counts) == set(src_files) and
            all(v == 1 for v in fresh_counts.values()))
    report["checks"]["C_fresh_banner_bijection"] = c_ok

    contamination_free = a_ok and b_ok and c_ok
    report["contamination_free"] = contamination_free

    # working-tree status vs fresh (informational: is WT already regenerated?)
    report["checks"]["working_tree_equals_fresh"] = (wt_b3 == fresh_b3)

    if contamination_free:
        report["standing"] = "ADMITTED"
        report["next_action"] = "proceed to render/score gate"
    else:
        report["standing"] = "PARTIAL_ALIVE"
        delta = report["head_banner_count"]
        report["next_action"] = (
            "R2b: triage the HEAD all_merged.ttl delta (HEAD banners=%d, "
            "fresh banners=%d) into ontology/source_law/*.ttl then regenerate"
            % (report["head_banner_count"], report["fresh_banner_count"]))

    _emit(report)
    return 0 if contamination_free else 1


def _emit(report):
    with open(os.path.join(REPO, "SOURCE_LAW_REPLAY_REPORT.json"), "w") as f:
        json.dump(report, f, indent=2, sort_keys=True)
    md = []
    md.append("# SOURCE_LAW_REPLAY_REPORT (R2a)\n")
    md.append("- timestamp: %s" % report.get("timestamp_utc"))
    md.append("- standing: **%s**" % report.get("standing", "UNKNOWN"))
    md.append("- contamination_free: %s" % report.get("contamination_free"))
    md.append("- head_merged_b3: `%s`" % report.get("head_merged_b3"))
    md.append("- fresh_merge_b3: `%s`" % report.get("fresh_merge_b3"))
    md.append("- working_tree_merged_b3: `%s`" % report.get("working_tree_merged_b3"))
    md.append("- head_banner_count: %s" % report.get("head_banner_count"))
    md.append("- fresh_banner_count: %s" % report.get("fresh_banner_count"))
    md.append("- source_law_count: %s" % report.get("source_law_count"))
    md.append("\n## Checks")
    for k, v in report.get("checks", {}).items():
        md.append("- %s: %s" % (k, v))
    if report.get("fatal"):
        md.append("\n## FATAL\n%s" % report["fatal"])
    md.append("\n## next_action\n%s" % report.get("next_action", ""))
    md.append("\n## source_law receipts (BLAKE3)")
    for f, h in sorted(report.get("source_law_receipts", {}).items()):
        md.append("- `%s`  %s" % (h, f))
    with open(os.path.join(REPO, "SOURCE_LAW_REPLAY_REPORT.md"), "w") as f:
        f.write("\n".join(md) + "\n")


if __name__ == "__main__":
    sys.exit(main())
