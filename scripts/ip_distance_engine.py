import os
import json
import glob
import hashlib
from datetime import datetime, timezone

def evaluate_candidates():
    policy_path = "/Users/sac/rocket-craft/ip_policy_packs/mecha_external_corpus.policy.json"
    if not os.path.exists(policy_path):
        print(f"Policy pack not found at {policy_path}")
        return

    with open(policy_path, "r") as f:
        policy = json.load(f)

    # Output directories
    output_dir = "/Users/sac/rocket-craft/generated/ip_distance_engine"
    os.makedirs(output_dir, exist_ok=True)

    ip_dist_report = {
        "engine": "Agent 17 IP Distance / Non-Confusion Agent",
        "policy_id": policy["policy_id"],
        "evaluations": []
    }

    non_confusion_report = {
        "status": "COMPLETED",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "candidates_evaluated": 0,
        "refused": 0,
        "admitted": 0,
        "details": []
    }
    
    receipts = []
    
    # We will search for all ttl files as candidates
    search_path = "/Users/sac/rocket-craft/generated/**/*.ttl"
    candidates = glob.glob(search_path, recursive=True)

    for candidate in candidates:
        non_confusion_report["candidates_evaluated"] += 1
        with open(candidate, "r", encoding="utf-8") as f:
            content = f.read().lower()
        
        filename = os.path.basename(candidate).lower()
        
        # Risk factors
        proximity_score = 0.0
        refusal_reasons = []
        
        # Check against protected clusters
        for cluster in policy["protected_clusters"]:
            cluster_name = cluster["franchise"].lower()
            if cluster_name in filename or cluster_name in content:
                proximity_score += 0.8 # High risk
                refusal_reasons.append(f"Protected source identifier '{cluster_name}' found.")
            
            for signature in cluster["signatures"]:
                # simple keyword matching for demo purposes
                keywords = [k.strip().lower() for k in signature.split() if len(k) > 4]
                for kw in keywords:
                    if kw in content:
                        proximity_score += 0.1
                        if proximity_score >= cluster["trade_dress_threshold"]:
                            refusal_reasons.append(f"External expressive-cluster proximity above threshold ({proximity_score:.2f}) for signature: {signature}")
                            break

        # Check provenance
        if "rip" in content or "scan" in content or "in the style of" in content:
            refusal_reasons.append("Prohibited provenance detected (scanned or ripped assets).")
            proximity_score += 1.0
            
        status = "REFUSED" if refusal_reasons else "ADMITTED"
        if status == "REFUSED":
            non_confusion_report["refused"] += 1
        else:
            non_confusion_report["admitted"] += 1
            
        eval_record = {
            "candidate": candidate,
            "status": status,
            "proximity_score": min(proximity_score, 1.0),
            "refusal_reasons": list(set(refusal_reasons))
        }
        ip_dist_report["evaluations"].append(eval_record)
        non_confusion_report["details"].append(eval_record)
        
        # Receipt
        receipt = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "candidate": candidate,
            "status": status,
            "hash": hashlib.blake2b(content.encode('utf-8')).hexdigest()
        }
        receipts.append(json.dumps(receipt))

    # Write output reports
    with open(os.path.join(output_dir, "IP_DISTANCE_REPORT.json"), "w") as f:
        json.dump(ip_dist_report, f, indent=2)
        
    with open(os.path.join(output_dir, "NON_CONFUSION_REPORT.json"), "w") as f:
        json.dump(non_confusion_report, f, indent=2)
        
    with open(os.path.join(output_dir, "ADMISSION_RECEIPT.jsonl"), "w") as f:
        f.write("\n".join(receipts))
        
    print(f"Evaluation complete. {non_confusion_report['candidates_evaluated']} candidates evaluated.")
    print(f"Admitted: {non_confusion_report['admitted']}, Refused: {non_confusion_report['refused']}")
    
if __name__ == "__main__":
    evaluate_candidates()
