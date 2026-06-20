import os
import json
import glob
import blake3
from datetime import datetime, timezone

# BaselineUsage law (097): external corpora are usable_for_metric_baseline=true,
# usable_for_generation=false. This block is required by the 099 schema.
BASELINE_USAGE = {
    "id": "ipd:MetricBaselineOnly",
    "usable_for_metric_baseline": True,
    "usable_for_generation": False,
}

# Admission rule (098): proximity at/above 0.7 -> REFUSE_EXPRESSIVE_PROXIMITY.
GLOBAL_ADMISSION_THRESHOLD = 0.7

HERO_CANDIDATE_BASENAME = "asset_fabric.ttl"


def b3(text):
    return blake3.blake3(text.encode("utf-8")).hexdigest()


def evaluate_candidates():
    policy_path = "/Users/sac/rocket-craft/ip_policy_packs/mecha_external_corpus.policy.json"
    if not os.path.exists(policy_path):
        print(f"Policy pack not found at {policy_path}")
        return

    with open(policy_path, "r") as f:
        policy = json.load(f)

    output_dir = "/Users/sac/rocket-craft/generated/ip_distance_engine"
    os.makedirs(output_dir, exist_ok=True)

    ip_dist_report = {
        "engine": "Agent 17 IP Distance / Non-Confusion Agent",
        "policy_id": policy["policy_id"],
        "hash_algorithm": "blake3",
        "baseline_usage": BASELINE_USAGE,
        "evaluations": [],
    }

    non_confusion_report = {
        "status": "COMPLETED",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "hash_algorithm": "blake3",
        "baseline_usage": BASELINE_USAGE,
        "candidates_evaluated": 0,
        "refused": 0,
        "admitted": 0,
        "verdict": "ADMIT_ORIGINAL",
        "details": [],
    }

    receipts = []

    search_path = "/Users/sac/rocket-craft/generated/**/*.ttl"
    candidates = sorted(glob.glob(search_path, recursive=True))

    for candidate in candidates:
        non_confusion_report["candidates_evaluated"] += 1
        with open(candidate, "r", encoding="utf-8") as f:
            raw = f.read()
        content = raw.lower()
        filename = os.path.basename(candidate).lower()

        proximity_score = 0.0
        refusal_reasons = []
        per_cluster = []

        for cluster in policy["protected_clusters"]:
            cluster_name = cluster["franchise"].lower()
            cluster_threshold = float(cluster["trade_dress_threshold"])
            cluster_score = 0.0

            if cluster_name in filename or cluster_name in content:
                cluster_score += 0.8
                refusal_reasons.append(
                    f"Protected source identifier '{cluster_name}' found."
                )

            for signature in cluster["signatures"]:
                keywords = [k.strip().lower() for k in signature.split() if len(k) > 4]
                for kw in keywords:
                    if kw in content:
                        cluster_score += 0.1

            proximity_score = max(proximity_score, cluster_score)
            per_cluster.append(
                {
                    "cluster": cluster["franchise"],
                    "proximity_score": round(min(cluster_score, 1.0), 4),
                    "trade_dress_threshold": cluster_threshold,
                }
            )

            if cluster_score >= cluster_threshold:
                refusal_reasons.append(
                    f"External expressive-cluster proximity {cluster_score:.2f} "
                    f">= trade_dress_threshold {cluster_threshold} for cluster "
                    f"'{cluster['franchise']}'."
                )

        # Prohibited provenance check (098)
        provenance_refused = False
        if "in the style of" in content or "ripped" in content or "traced from" in content:
            refusal_reasons.append(
                "Prohibited provenance detected (scanned/ripped/traced assets)."
            )
            proximity_score = max(proximity_score, 1.0)
            provenance_refused = True

        proximity_score = min(proximity_score, 1.0)

        # Verdict mapping to 098 AdmissionVerdict individuals
        if provenance_refused:
            verdict = "REFUSE_PROHIBITED_PROVENANCE"
        elif proximity_score >= GLOBAL_ADMISSION_THRESHOLD or refusal_reasons:
            verdict = "REFUSE_EXPRESSIVE_PROXIMITY"
        else:
            verdict = "ADMIT_ORIGINAL"

        status = "ADMITTED" if verdict == "ADMIT_ORIGINAL" else "REFUSED"
        if status == "REFUSED":
            non_confusion_report["refused"] += 1
        else:
            non_confusion_report["admitted"] += 1

        is_hero = os.path.basename(candidate) == HERO_CANDIDATE_BASENAME

        eval_record = {
            "candidate": candidate,
            "candidate_id": os.path.basename(candidate),
            "is_hero": is_hero,
            "status": status,
            "verdict": verdict,
            "proximity_score": round(proximity_score, 4),
            "per_cluster_proximity": per_cluster,
            "baseline_usage": BASELINE_USAGE,
            "refusal_reasons": sorted(set(refusal_reasons)),
        }
        ip_dist_report["evaluations"].append(eval_record)
        non_confusion_report["details"].append(eval_record)

        receipt = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "candidate_id": os.path.basename(candidate),
            "candidate": candidate,
            "status": status,
            "verdict": verdict,
            "proximity_score": round(proximity_score, 4),
            "baseline_usage": BASELINE_USAGE,
            "hash_algorithm": "blake3",
            "blake3_hash": b3(raw),
        }
        receipts.append(json.dumps(receipt))

    # Overall non-confusion verdict: REFUSE if any candidate refused
    if non_confusion_report["refused"] > 0:
        non_confusion_report["verdict"] = "REFUSE_EXPRESSIVE_PROXIMITY"
    else:
        non_confusion_report["verdict"] = "ADMIT_ORIGINAL"

    # Serialize, then stamp a self-hash (blake3) over the payload
    ip_path = os.path.join(output_dir, "IP_DISTANCE_REPORT.json")
    nc_path = os.path.join(output_dir, "NON_CONFUSION_REPORT.json")
    rc_path = os.path.join(output_dir, "ADMISSION_RECEIPT.jsonl")

    ip_dist_report["blake3_hash"] = b3(json.dumps(ip_dist_report, sort_keys=True))
    non_confusion_report["blake3_hash"] = b3(
        json.dumps(non_confusion_report, sort_keys=True)
    )

    with open(ip_path, "w") as f:
        json.dump(ip_dist_report, f, indent=2)
    with open(nc_path, "w") as f:
        json.dump(non_confusion_report, f, indent=2)
    with open(rc_path, "w") as f:
        f.write("\n".join(receipts) + ("\n" if receipts else ""))

    print(
        f"Evaluation complete. {non_confusion_report['candidates_evaluated']} "
        f"candidates evaluated."
    )
    print(
        f"Admitted: {non_confusion_report['admitted']}, "
        f"Refused: {non_confusion_report['refused']}, "
        f"Verdict: {non_confusion_report['verdict']}"
    )


if __name__ == "__main__":
    evaluate_candidates()
