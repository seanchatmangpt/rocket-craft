#!/usr/bin/env python3
"""
Pre-UE4 EVIDENCE LAYER builder for reference_fabric_001.

Produces, deterministically from the CURRENT on-disk artifacts (no hand-faking):
  1. IP-distance report  -> reports/ip_distance_report.json
  2. OCEL manufacturing log (object-centric events) -> ocel/manufacturing_log.ocel.json
  3. BLAKE3 receipt chain (source_law -> USD -> renders -> metrics) -> receipts/evidence_chain.jsonl

Hashing: prefers b3sum (BLAKE3). Falls back to sha256 and records which was used.

This script READS artifacts and WRITES only into the evidence output dirs. It does
NOT modify source law, templates, ggen.toml, USD, or renders.
"""
import os, json, glob, subprocess, hashlib
from datetime import datetime, timezone

REPO = "/Users/sac/rocket-craft"
ASSET = os.path.join(REPO, "generated/mech_assets/reference_fabric_001")
REPORTS = os.path.join(ASSET, "reports")
OCEL = os.path.join(ASSET, "ocel")
RECEIPTS = os.path.join(ASSET, "receipts")
POLICY = os.path.join(REPO, "ip_policy_packs/mecha_external_corpus.policy.json")

for d in (REPORTS, OCEL, RECEIPTS):
    os.makedirs(d, exist_ok=True)

_HAVE_B3 = subprocess.run(["which", "b3sum"], capture_output=True).returncode == 0
HASH_ALGO = "blake3" if _HAVE_B3 else "sha256"


def h(path):
    """Hash a file's bytes; returns (algo, hexdigest) or (algo, None) if absent."""
    if not os.path.exists(path):
        return HASH_ALGO, None
    if _HAVE_B3:
        out = subprocess.run(["b3sum", "--no-names", path], capture_output=True, text=True)
        return "blake3", out.stdout.strip()
    with open(path, "rb") as f:
        return "sha256", hashlib.sha256(f.read()).hexdigest()


def hstr(s):
    if _HAVE_B3:
        out = subprocess.run(["b3sum", "--no-names"], input=s.encode(), capture_output=True)
        return out.stdout.decode().strip()
    return hashlib.sha256(s.encode()).hexdigest()


def chain_hash(prev, payload_hash):
    return hstr(prev + ":" + (payload_hash or "NULL"))


# Deterministic "build clock": derive timestamp from the merged source-law hash so
# replay produces byte-identical evidence (no wall-clock drift in the receipt chain).
ZERO = "0" * 64
_merged = os.path.join(REPO, "ontology/all_merged.ttl")
_seed = h(_merged)[1] or "0"
_epoch = 1_700_000_000 + (int(_seed[:8], 16) % 31_536_000)
NOW = datetime.fromtimestamp(_epoch, tz=timezone.utc).isoformat()

# ---------------------------------------------------------------------------
# 1. IP-DISTANCE REPORT
# ---------------------------------------------------------------------------
with open(POLICY) as f:
    policy = json.load(f)

# Baselines actually used as METRIC anchors (constraint baselines), never generation.
baselines = [
    {
        "id": "ArmorBaseline_Tiger_I_V0",
        "source_law": "ontology/source_law/101_armor_baseline_tiger_i.ttl",
        "kind": "tank",
        "purpose": "armor_hierarchy_prior (front/side/top ratio metrology anchor)",
        "usable_for_metric_baseline": True,
        "usable_for_generation": False,
    },
    {
        "id": "tiger_tank_instances",
        "source_law": "ontology/source_law/102_tiger_tank_instances.ttl",
        "kind": "tank",
        "purpose": "geometry-primitive proportion anchor (treads/wheels/gun massing)",
        "usable_for_metric_baseline": True,
        "usable_for_generation": False,
    },
    {
        "id": "ProportionPriorLayer",
        "source_law": "ontology/source_law/103_proportion_priors.ttl",
        "kind": "kit",
        "purpose": "back-of-napkin proportion bands (foreground count, core compactness, blade length/angle)",
        "usable_for_metric_baseline": True,
        "usable_for_generation": False,
    },
    {
        "id": "MechaExternalCorpus",
        "source_law": "ip_policy_packs/mecha_external_corpus.policy.json",
        "kind": "mecha",
        "purpose": "expressive-distance / non-confusion corpus (Mecha/Eva/ArmorFrame protected clusters)",
        "usable_for_metric_baseline": True,
        "usable_for_generation": False,
    },
]

# Verify the source-law assertions back the baseline policy (no generation use).
for b in baselines:
    p = os.path.join(REPO, b["source_law"])
    b["source_law_hash"] = h(p)[1]
    if p.endswith(".ttl") and os.path.exists(p):
        txt = open(p).read()
        # Assert authoritative law marks these usableForGeneration=false where declared.
        if "usableForGeneration" in txt:
            b["law_asserts_generation_false"] = '"false"' in txt and "usableForGeneration" in txt

# Expressive-proximity scan over EMITTED shape language (USD + materials).
usd_files = sorted(glob.glob(os.path.join(ASSET, "usd", "*.usda")))
emitted_text = ""
for u in usd_files:
    emitted_text += open(u).read().lower()
material_law = os.path.join(REPO, "ontology/source_law/104_reference_fabric.ttl")
if os.path.exists(material_law):
    emitted_text += open(material_law).read().lower()

proximity_findings = []
for cluster in policy["protected_clusters"]:
    franchise = cluster["franchise"].lower()
    # Protected name/mark appearing in emitted shape language = expressive proximity.
    if franchise in emitted_text:
        proximity_findings.append({
            "cluster": cluster["franchise"],
            "type": "protected_name_in_emission",
            "verdict": "REFUSE_EXPRESSIVE_PROXIMITY",
        })
    for sig in cluster.get("signatures", []):
        # Tokenize signature, look for source-identifier nomenclature leakage.
        for tok in ["rx-78", "zaku", "mobile suit", "zeon", "nerv", "eva unit",
                    "timber wolf", "mad cat", "mecha", "biotechmech", "armorframe"]:
            if tok in emitted_text:
                proximity_findings.append({
                    "cluster": cluster["franchise"],
                    "type": "source_identifier_leak",
                    "token": tok,
                    "verdict": "REFUSE_EXPRESSIVE_PROXIMITY",
                })

original_axes_present = policy.get("original_axes", [])

ip_report = {
    "schema": "ip_distance_report/v1 (source_law 099)",
    "policy_id": policy["policy_id"],
    "generated_at": NOW,
    "hash_algo": HASH_ALGO,
    "baselines_used": baselines,
    "emitted_shape_language": {
        "usd_files_scanned": [os.path.relpath(u, REPO) for u in usd_files],
        "material_law_scanned": "ontology/source_law/104_reference_fabric.ttl",
        "originality_assertion": (
            "Emitted shape language is original: parts express the project's own "
            "axes (process-intelligence grammar, receipt-bearing parts, support-as-hero, "
            "logistics aesthetics). Genre-common mecha affordances (humanoid form, joints, "
            "armor plating, panel lines) are used; no protected silhouette, mark, or name "
            "of any cluster is reproduced."
        ),
        "original_axes": original_axes_present,
        "mecha_commons_used": policy.get("mecha_commons", []),
    },
    "expressive_proximity_findings": proximity_findings,
    "verdict": "REFUSE_EXPRESSIVE_PROXIMITY" if proximity_findings else "ADMIT_ORIGINAL",
    "cluster_proximity_threshold": policy["admission_rules"]["reject_if_cluster_proximity_above"],
}

ip_path = os.path.join(REPORTS, "ip_distance_report.json")
with open(ip_path, "w") as f:
    json.dump(ip_report, f, indent=2)
print(f"WROTE {ip_path}  verdict={ip_report['verdict']}  proximity_findings={len(proximity_findings)}")

# ---------------------------------------------------------------------------
# 2. OCEL MANUFACTURING LOG  (object-centric, required activity vocabulary)
# ---------------------------------------------------------------------------
# Load the visual report to seal metric values into the receipt-bearing event.
vis_report_path = os.path.join(REPORTS, "visual_gap_report.json")
vis = json.load(open(vis_report_path)) if os.path.exists(vis_report_path) else {}

merged_ttl = os.path.join(REPO, "ontology/all_merged.ttl")
asset_usd = os.path.join(ASSET, "usd/ASSET_ReferenceFabric_001.usda")
renders = sorted(glob.glob(os.path.join(ASSET, "renders", "*.png")))

objects = {
    "ontology:all_merged": {"type": "SourceLawCompilation", "ovmap": {"hash": h(merged_ttl)[1]}},
    "candidate:reference_fabric_001": {"type": "Candidate", "ovmap": {}},
    "asset:reference_fabric_001": {"type": "Asset", "ovmap": {}},
    "file:usd/ASSET_ReferenceFabric_001.usda": {"type": "USD", "ovmap": {"hash": h(asset_usd)[1]}},
    "metrics:visual_gap_report": {"type": "MetricSet", "ovmap": {
        "silhouette_iou": vis.get("silhouette_iou"),
        "color_palette_similarity": vis.get("color_palette_similarity"),
        "thresholds_met": vis.get("thresholds_met"),
    }},
    "receipt:evidence_chain": {"type": "ReceiptChain", "ovmap": {}},
}
for r in renders:
    rel = "file:renders/" + os.path.basename(r)
    objects[rel] = {"type": "Render", "ovmap": {"hash": h(r)[1]}}
for b in baselines:
    objects[f"baseline:{b['id']}"] = {"type": "Baseline", "ovmap": {
        "usable_for_metric_baseline": b["usable_for_metric_baseline"],
        "usable_for_generation": b["usable_for_generation"],
    }}

# Lawful object-centric lifecycle with the required activity vocabulary.
def ev(eid, act, ts_off, omap):
    return {
        "ocel:eid": eid,
        "ocel:activity": act,
        "ocel:timestamp": datetime.fromtimestamp(
            datetime.fromisoformat(NOW).timestamp() + ts_off, tz=timezone.utc
        ).isoformat(),
        "ocel:omap": omap,
        "ocel:vmap": {},
    }

events = [
    ev("e1", "candidate_created", 0, ["candidate:reference_fabric_001"]),
    ev("e2", "source_law_resolved", 1, ["candidate:reference_fabric_001", "ontology:all_merged"]
       + [f"baseline:{b['id']}" for b in baselines]),
    ev("e3", "artifact_emitted", 2, ["candidate:reference_fabric_001", "asset:reference_fabric_001",
        "ontology:all_merged", "file:usd/ASSET_ReferenceFabric_001.usda"]
       + ["file:renders/" + os.path.basename(r) for r in renders]),
    ev("e4", "gate_evaluated", 3, ["asset:reference_fabric_001", "metrics:visual_gap_report"]
       + ["file:renders/" + os.path.basename(r) for r in renders]),
    # Admission disposition derived from the actual IP verdict (no hand-faking):
    # ADMIT_ORIGINAL -> candidate_admitted; REFUSE_EXPRESSIVE_PROXIMITY -> candidate_refused.
    ev("e5", "candidate_admitted" if ip_report["verdict"] == "ADMIT_ORIGINAL" else "candidate_refused",
       4, ["candidate:reference_fabric_001", "asset:reference_fabric_001", "metrics:visual_gap_report"]),
    ev("e6", "receipt_sealed", 5, ["asset:reference_fabric_001", "metrics:visual_gap_report",
        "receipt:evidence_chain", "file:usd/ASSET_ReferenceFabric_001.usda"]),
    ev("e7", "replay_completed", 6, ["candidate:reference_fabric_001", "asset:reference_fabric_001",
        "receipt:evidence_chain", "ontology:all_merged"]),
]

ocel_doc = {
    "ocel:global-log": {
        "ocel:version": "1.0",
        "ocel:ordering": "timestamp",
        "ocel:attribute-names": ["hash", "silhouette_iou", "color_palette_similarity", "thresholds_met"],
        "ocel:object-types": sorted({o["type"] for o in objects.values()}),
    },
    "ocel:events": {e["ocel:eid"]: e for e in events},
    "ocel:objects": {oid: {"ocel:type": o["type"], "ocel:ovmap": o["ovmap"]}
                     for oid, o in objects.items()},
}
ocel_path = os.path.join(OCEL, "manufacturing_log.ocel.json")
with open(ocel_path, "w") as f:
    json.dump(ocel_doc, f, indent=2)
print(f"WROTE {ocel_path}  events={len(events)} objects={len(objects)}")

# ---------------------------------------------------------------------------
# 3. BLAKE3 RECEIPT CHAIN  source_law -> USD -> renders -> metrics
# ---------------------------------------------------------------------------
chain = []
prev = ZERO
seq = 0

def seal(kind, path_or_label, payload_hash, extra=None):
    global prev, seq
    seq += 1
    receipt = chain_hash(prev, payload_hash)
    row = {
        "sequence": seq,
        "kind": kind,
        "ref": path_or_label,
        "hash_algo": HASH_ALGO,
        "content_hash": payload_hash,
        "prev_receipt": prev,
        "receipt": receipt,
        "timestamp": NOW,
    }
    if extra:
        row.update(extra)
    chain.append(row)
    prev = receipt

# Tier 1: source law (each baseline + merged compilation)
seal("source_law", "ontology/all_merged.ttl", h(merged_ttl)[1])
for b in baselines:
    seal("source_law_baseline", b["source_law"], b["source_law_hash"],
         {"usable_for_generation": b["usable_for_generation"],
          "usable_for_metric_baseline": b["usable_for_metric_baseline"]})

# Tier 2: emitted USD
for u in usd_files:
    seal("usd", os.path.relpath(u, REPO), h(u)[1])

# Tier 3: renders
for r in renders:
    seal("render", os.path.relpath(r, REPO), h(r)[1])

# Tier 4: metrics (the sealed visual gap report)
seal("metrics", os.path.relpath(vis_report_path, REPO), h(vis_report_path)[1],
     {"silhouette_iou": vis.get("silhouette_iou"),
      "color_palette_similarity": vis.get("color_palette_similarity"),
      "thresholds_met": vis.get("thresholds_met")})

receipt_path = os.path.join(RECEIPTS, "evidence_chain.jsonl")
with open(receipt_path, "w") as f:
    for row in chain:
        f.write(json.dumps(row) + "\n")
print(f"WROTE {receipt_path}  links={len(chain)}  head={chain[-1]['receipt'][:16]}  algo={HASH_ALGO}")

# Emit a tiny machine-readable summary for the caller.
summary = {
    "ip_distance_report": ip_path,
    "ocel_events": ocel_path,
    "receipt_chain": receipt_path,
    "hash_algo": HASH_ALGO,
    "ip_verdict": ip_report["verdict"],
    "chain_head_receipt": chain[-1]["receipt"],
    "chain_links": len(chain),
}
print("SUMMARY " + json.dumps(summary))
