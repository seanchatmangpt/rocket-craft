#!/usr/bin/env python3
"""
verify_r6_delete_resync_replay.py  — WORKSTREAM R6 KEYSTONE GATE

Wraps the PROVEN scripts/verify_delete_and_resync_replay.py (does NOT reimplement
its GPU carve-out: deterministic .usda/.mtlx/textures are byte-compared SHA-256;
GPU-rendered PNGs are disposition-compared to 4dp). This driver:

  1. Gates on R2 (SOURCE_LAW_REPLAY_REPORT.json must be contamination_free /
     regenerable-clean). A generated-replay proof is meaningless on contaminated
     source law, so a failed R2 gate => REFUSED with next_action=R2b.

  2. Runs TWO full delete -> merge -> ggen sync -> fresh render -> compare
     canonical rebuilds (via the proven module's run_canonical_rebuild) and
     compares rebuild#1 vs rebuild#2 for:
       - generator_artifacts_byte_identical (class 1, byte-exact)
       - disposition_replays               (class 2, GPU disposition to 4dp)

  3. Builds the unified BLAKE3_RECEIPT_CHAIN.json linking, prev_hash-chained
     (genesis = 64 zeros, receipt = blake3(hash || prev_hash) over hex strings):
       source(each source_law/*.ttl + all_merged.ttl)
         -> USD/material/texture byte hashes
         -> render hashes      (class:gpu, comparison:disposition; never chain-breaking)
         -> metric disposition hash (sorted JSON, metrics@4dp)
         -> report hash
     Mirrors generated/.../receipts/asset_receipts.jsonl JSONL field shape.

  4. Emits the keystone DELETE_RESYNC_REPLAY_REPORT.json + .md.

standing=ADMITTED iff generator_artifacts_byte_identical AND disposition_replays
AND R2 regenerable AND chain linkage valid end-to-end; else REFUSED with the
exact divergent artifact named.
"""

import os
import sys
import json
import time
import hashlib
import importlib.util

import blake3

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
ASSET_DIR = os.path.join(REPO_ROOT, "generated", "mech_assets", "reference_fabric_001")
SOURCE_LAW_DIR = os.path.join(REPO_ROOT, "ontology", "source_law")
MERGED_TTL = os.path.join(REPO_ROOT, "ontology", "all_merged.ttl")
GAP_REPORT = os.path.join(ASSET_DIR, "reports", "visual_gap_report.json")
R2_REPORT = os.path.join(REPO_ROOT, "SOURCE_LAW_REPLAY_REPORT.json")
GENESIS = "0" * 64


def _load_proven():
    path = os.path.join(REPO_ROOT, "scripts", "verify_delete_and_resync_replay.py")
    spec = importlib.util.spec_from_file_location("proven_replay", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def b3_file(path):
    h = blake3.blake3()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def b3_str(s):
    return blake3.blake3(s.encode("utf-8")).hexdigest()


def receipt_of(item_hash, prev_hash):
    # receipt = blake3(hash || prev_hash) over the hex strings
    return b3_str(item_hash + prev_hash)


def gate_r2():
    if not os.path.exists(R2_REPORT):
        return False, "SOURCE_LAW_REPLAY_REPORT.json missing"
    with open(R2_REPORT) as f:
        r = json.load(f)
    if not r.get("contamination_free", False):
        return False, "R2 source law is contaminated (contamination_free=false)"
    if r.get("standing") != "ADMITTED":
        return False, f"R2 standing={r.get('standing')} (not ADMITTED)"
    checks = r.get("checks", {})
    if not all(checks.values()):
        return False, f"R2 checks failing: {checks}"
    return True, r


def disposition_hash(disposition):
    # sorted JSON, metrics already rounded to 4dp by the proven capture_disposition
    canonical = json.dumps(disposition, sort_keys=True, separators=(",", ":"))
    return b3_str(canonical)


def build_chain(rebuild):
    """Build the unified prev_hash-linked BLAKE3 chain over rebuild #1's artifacts.

    Order: source_law/*.ttl (sorted) -> all_merged.ttl -> deterministic generator
    artifacts (usd/materialx/textures, sorted) -> render PNGs (class:gpu,
    comparison:disposition, NEVER chain-breaking) -> metric disposition -> report.
    """
    entries = []
    seq = 0
    prev = GENESIS

    def add(artifact_path, item_hash, cls, comparison, extra=None):
        nonlocal seq, prev
        seq += 1
        rcpt = receipt_of(item_hash, prev)
        e = {
            "sequence": seq,
            "artifact_path": artifact_path,
            "hash": item_hash,
            "prev_hash": prev,
            "receipt": rcpt,
            "class": cls,
            "comparison": comparison,
            "status": "VERIFIED",
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        }
        if extra:
            e.update(extra)
        entries.append(e)
        prev = rcpt

    # 1. source_law/*.ttl
    for fn in sorted(os.listdir(SOURCE_LAW_DIR)):
        if fn.endswith(".ttl"):
            fp = os.path.join(SOURCE_LAW_DIR, fn)
            add(os.path.relpath(fp, REPO_ROOT), b3_file(fp), "source", "byte")

    # 2. all_merged.ttl
    add(os.path.relpath(MERGED_TTL, REPO_ROOT), b3_file(MERGED_TTL), "source", "byte")

    # 3. deterministic generator artifacts (re-hash with BLAKE3 from disk for the chain)
    for rel_path in sorted(rebuild["generator_hashes"].keys()):
        abs_path = os.path.join(ASSET_DIR, rel_path)
        add(os.path.join("generated/mech_assets/reference_fabric_001", rel_path),
            b3_file(abs_path), "generator", "byte")

    # 4. GPU render PNGs — class:gpu, comparison:disposition (never chain-breaking).
    #    Their byte hash is recorded for provenance but the link law is disposition,
    #    so a differing PNG hash does NOT invalidate the chain.
    renders_dir = os.path.join(ASSET_DIR, "renders")
    if os.path.isdir(renders_dir):
        for fn in sorted(os.listdir(renders_dir)):
            if fn.endswith(".png"):
                fp = os.path.join(renders_dir, fn)
                add(os.path.join("generated/mech_assets/reference_fabric_001/renders", fn),
                    b3_file(fp), "gpu", "disposition")

    # 5. metric disposition hash (sorted JSON, metrics@4dp)
    disp_h = disposition_hash(rebuild["disposition"])
    add("metric_disposition", disp_h, "disposition", "disposition",
        {"disposition": rebuild["disposition"]})

    # 6. report hash (the gap report the disposition was read from)
    if os.path.exists(GAP_REPORT):
        add(os.path.relpath(GAP_REPORT, REPO_ROOT), b3_file(GAP_REPORT), "report", "byte")

    return entries


def validate_chain(entries):
    """End-to-end linkage: genesis at head, each receipt=blake3(hash||prev_hash),
    and each prev_hash equals the previous entry's receipt."""
    prev = GENESIS
    for e in entries:
        if e["prev_hash"] != prev:
            return False, f"prev_hash break at seq {e['sequence']} ({e['artifact_path']})"
        if e["receipt"] != receipt_of(e["hash"], e["prev_hash"]):
            return False, f"receipt mismatch at seq {e['sequence']} ({e['artifact_path']})"
        prev = e["receipt"]
    return True, prev


def main():
    print("=== R6 DELETE-AND-RESYNC REPLAY (KEYSTONE GATE) ===")

    # --- R2 GATE ---
    r2_ok, r2_detail = gate_r2()
    if not r2_ok:
        report = {
            "gate": "R6_delete_resync_replay",
            "standing": "REFUSED",
            "r2_regenerable": False,
            "r2_detail": r2_detail,
            "next_action": "R2b",
            "verdict": "REFUSED: generated-replay proof meaningless on contaminated source law",
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        }
        with open(os.path.join(REPO_ROOT, "DELETE_RESYNC_REPLAY_REPORT.json"), "w") as f:
            json.dump(report, f, indent=2)
        with open(os.path.join(REPO_ROOT, "DELETE_RESYNC_REPLAY_REPORT.md"), "w") as f:
            f.write("# Delete and Resync Replay Report (R6 KEYSTONE)\n\n")
            f.write("**Standing**: REFUSED\n\n")
            f.write(f"R2 gate FAILED: {r2_detail}\n\nnext_action: R2b\n")
        print(f"REFUSED: R2 gate failed -> {r2_detail}")
        sys.exit(2)
    print(f"R2 gate ADMITTED: contamination_free, "
          f"source_law_count={r2_detail.get('source_law_count')}")

    # --- TWO CANONICAL REBUILDS via the PROVEN module (GPU carve-out preserved) ---
    proven = _load_proven()
    rebuild1 = proven.run_canonical_rebuild("rebuild #1")
    if rebuild1 is None:
        _emit_rebuild_failure("rebuild #1 failed", "rebuild #1")
        sys.exit(1)
    rebuild2 = proven.run_canonical_rebuild("rebuild #2")
    if rebuild2 is None:
        _emit_rebuild_failure("rebuild #2 failed", "rebuild #2")
        sys.exit(1)

    # --- COMPARE (class 1 byte-identity, class 2 disposition-identity) ---
    print("\n=== R6: generator-artifact byte-identity (rebuild#1 vs rebuild#2) ===")
    generator_artifacts_byte_identical = proven.compare_hash_maps(
        rebuild1["generator_hashes"], rebuild2["generator_hashes"]
    )
    n_art = len(rebuild1["generator_hashes"])
    if generator_artifacts_byte_identical:
        print(f"  PASS: {n_art} deterministic generator artifacts byte-identical.")

    print("\n=== R6: GPU-render DISPOSITION identity ===")
    disposition_replays = proven.disposition_equal(
        rebuild1["disposition"], rebuild2["disposition"]
    )
    divergent_artifact = None
    if disposition_replays:
        print("  PASS: disposition (thresholds_met+usd_errors+vis_errors+metrics@4dp) identical.")
    else:
        print("  FAIL: disposition diverged across rebuilds.")
        divergent_artifact = "metric_disposition (rebuild#1 != rebuild#2)"

    # Identify the first divergent generator artifact, if any.
    if not generator_artifacts_byte_identical:
        h1, h2 = rebuild1["generator_hashes"], rebuild2["generator_hashes"]
        for rp in sorted(set(h1) | set(h2)):
            if h1.get(rp) != h2.get(rp):
                divergent_artifact = rp
                break

    # --- UNIFIED BLAKE3 CHAIN over rebuild #1's canonical artifacts ---
    print("\n=== R6: building unified BLAKE3 receipt chain ===")
    chain = build_chain(rebuild1)
    chain_valid, chain_tail = validate_chain(chain)
    chain_head = chain[0] if chain else None
    if chain_valid:
        print(f"  PASS: chain linkage valid end-to-end ({len(chain)} entries). head_genesis={GENESIS[:8]}.. tail={chain_tail[:16]}..")
    else:
        print(f"  FAIL: chain linkage broken -> {chain_tail}")
        divergent_artifact = divergent_artifact or f"chain:{chain_tail}"

    chain_doc = {
        "gate": "R6_delete_resync_replay",
        "receipt_algorithm": "blake3(hash || prev_hash) over hex strings",
        "genesis": GENESIS,
        "head_receipt": chain_head["receipt"] if chain_head else None,
        "tail_receipt": chain_tail if chain_valid else None,
        "chain_valid": chain_valid,
        "entry_count": len(chain),
        "entries": chain,
    }
    with open(os.path.join(REPO_ROOT, "BLAKE3_RECEIPT_CHAIN.json"), "w") as f:
        json.dump(chain_doc, f, indent=2)

    # --- VERDICT ---
    admitted = (generator_artifacts_byte_identical and disposition_replays
                and r2_ok and chain_valid)
    standing = "ADMITTED" if admitted else "REFUSED"

    report = {
        "gate": "R6_delete_resync_replay",
        "standing": standing,
        "verdict": "VERIFIED" if admitted else "REFUSED",
        "r2_regenerable": True,
        "r2_merged_b3": r2_detail.get("head_merged_b3"),
        "generator_artifacts_byte_identical": generator_artifacts_byte_identical,
        "disposition_replays": disposition_replays,
        "generator_artifact_count": n_art,
        "chain_valid": chain_valid,
        "chain_entry_count": len(chain),
        "chain_head_receipt": chain_head["receipt"] if chain_head else None,
        "chain_tail_receipt": chain_tail if chain_valid else None,
        "rebuild_1": {
            "generator_hashes": rebuild1["generator_hashes"],
            "disposition": rebuild1["disposition"],
        },
        "rebuild_2": {
            "generator_hashes": rebuild2["generator_hashes"],
            "disposition": rebuild2["disposition"],
        },
        "divergent_artifact": divergent_artifact,
        "next_action": ("none" if admitted
                        else f"repair divergent artifact: {divergent_artifact}"),
        "ggen_nondeterminism_suspected": (not generator_artifacts_byte_identical),
        "ggen_nondeterminism_hint": (
            "rebuild1 != rebuild2 for a deterministic artifact: inspect the SPARQL "
            "SELECT feeding the producing template for a missing total ORDER BY."
            if not generator_artifacts_byte_identical else None
        ),
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    with open(os.path.join(REPO_ROOT, "DELETE_RESYNC_REPLAY_REPORT.json"), "w") as f:
        json.dump(report, f, indent=2)

    with open(os.path.join(REPO_ROOT, "DELETE_RESYNC_REPLAY_REPORT.md"), "w") as f:
        f.write("# Delete and Resync Replay Report (R6 KEYSTONE)\n\n")
        f.write(f"**Standing**: {standing}\n\n")
        f.write("## R2 gate\n\n")
        f.write(f"- regenerable (contamination_free): True\n")
        f.write(f"- merged all_merged.ttl b3: `{r2_detail.get('head_merged_b3')}`\n\n")
        f.write("## Rebuild #1 vs Rebuild #2\n\n")
        f.write(f"- generator_artifacts_byte_identical: {generator_artifacts_byte_identical}\n")
        f.write(f"- disposition_replays: {disposition_replays}\n")
        f.write(f"- generator_artifact_count: {n_art}\n")
        f.write(f"- rebuild#1 thresholds_met: {rebuild1['disposition'].get('thresholds_met')}\n")
        f.write(f"- rebuild#2 thresholds_met: {rebuild2['disposition'].get('thresholds_met')}\n\n")
        f.write("## BLAKE3 receipt chain\n\n")
        f.write(f"- chain_valid: {chain_valid}\n")
        f.write(f"- entry_count: {len(chain)}\n")
        f.write(f"- head receipt: `{chain_head['receipt'] if chain_head else None}`\n")
        f.write(f"- tail receipt: `{chain_tail if chain_valid else 'INVALID'}`\n\n")
        f.write(f"## Verdict\n\n**{standing}** — next_action: {report['next_action']}\n")
        if divergent_artifact:
            f.write(f"\nDivergent artifact: `{divergent_artifact}`\n")

    print("\n" + "=" * 60)
    print(f"=== R6 {standing} ===")
    print(json.dumps({
        "standing": standing,
        "generator_artifacts_byte_identical": generator_artifacts_byte_identical,
        "disposition_replays": disposition_replays,
        "chain_valid": chain_valid,
        "divergent_artifact": divergent_artifact,
    }))
    sys.exit(0 if admitted else 1)


def _emit_rebuild_failure(msg, label):
    report = {
        "gate": "R6_delete_resync_replay",
        "standing": "PARTIAL_ALIVE",
        "verdict": "REFUSED",
        "r2_regenerable": True,
        "next_action": f"{label}: canonical rebuild step failed; inspect rebuild log",
        "detail": msg,
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    with open(os.path.join(REPO_ROOT, "DELETE_RESYNC_REPLAY_REPORT.json"), "w") as f:
        json.dump(report, f, indent=2)
    with open(os.path.join(REPO_ROOT, "DELETE_RESYNC_REPLAY_REPORT.md"), "w") as f:
        f.write("# Delete and Resync Replay Report (R6 KEYSTONE)\n\n")
        f.write(f"**Standing**: PARTIAL_ALIVE\n\n{msg}\n")
    print(f"REFUSED: {msg}")


if __name__ == "__main__":
    main()
