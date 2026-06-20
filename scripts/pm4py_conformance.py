#!/usr/bin/env python3
"""
pm4py_conformance.py — Van der Aalst process conformance check.

Reads an OCEL 2.0 JSON export from the Nuxt shell
(GET /api/game/ocel-export?session_id=<uuid>) and uses pm4py to:

  1. Parse the OCEL 2.0 log
  2. Discover the actual process model (inductive miner)
  3. Compute conformance metrics (fitness, precision, generalization)
  4. Verify the declared lawful lifecycle is a valid trace
  5. Write a fitness_report.json (consumed by anti-llm-cheat-lsp)

Van der Aalst doctrine:
  The pipeline is ALIVE only when the mined process model accepts
  the declared lifecycle as a fitting trace. Fitness < 1.0 means
  the actual execution deviated from the declared model.

Usage:
  # Export a session:
  curl "http://localhost:3000/api/game/ocel-export?session_id=<uuid>" > /tmp/session.ocel.json

  # Run conformance:
  python3 scripts/pm4py_conformance.py /tmp/session.ocel.json

  # Or pipe directly:
  curl "http://localhost:3000/api/game/ocel-export?session_id=<uuid>" | \\
    python3 scripts/pm4py_conformance.py -

Requirements:
  pip install pm4py

Output:
  Writes ocel/reports/<session_id>_fitness_report.json
  Exits 0 if fitness >= THRESHOLD (default 0.8), 1 otherwise.
"""

import json
import sys
import argparse
from pathlib import Path

DECLARED_LIFECYCLE = [
    "GameSessionStarted",
    "FrameRendered",
    "InputAdmitted",
]

FITNESS_THRESHOLD = 0.8  # minimum acceptable fitness to pass
REPORTS_DIR = Path(__file__).parent.parent / "nuxt-shell" / "ocel" / "reports"


def load_ocel2_json(path: str) -> dict:
    """Read OCEL 2.0 JSON from file path or stdin (-)."""
    if path == "-":
        return json.load(sys.stdin)
    with open(path) as f:
        return json.load(f)


def ocel2_to_pm4py(ocel2: dict):
    """
    Convert the server's OCEL 2.0 JSON (camelCase) to pm4py OCEL format.

    The server emits: { objectTypes, eventTypes, objects, events }
    pm4py read_ocel2_json expects the OCEL 2.0 standard XML-JSON with
    specific key naming. We convert manually to avoid format mismatch.
    """
    try:
        import pm4py
        from pm4py.objects.ocel.obj import OCEL
    except ImportError:
        print("ERROR: pm4py not installed. Run: pip install pm4py", file=sys.stderr)
        sys.exit(2)

    import pandas as pd

    events = ocel2.get("events", [])
    objects = ocel2.get("objects", [])

    if not events:
        raise ValueError("OCEL 2.0 log has no events")

    # Build events dataframe (pm4py OCEL requires specific column names)
    event_rows = []
    for evt in events:
        event_rows.append({
            "ocel:eid": evt.get("id", ""),
            "ocel:activity": evt.get("type", ""),
            "ocel:timestamp": pd.Timestamp(evt.get("time", "1970-01-01")),
        })
    events_df = pd.DataFrame(event_rows)

    # Build relations dataframe (event → object)
    relation_rows = []
    for evt in events:
        for rel in evt.get("relationships", []):
            relation_rows.append({
                "ocel:eid": evt.get("id", ""),
                "ocel:oid": rel.get("objectId", ""),
                "ocel:qualifier": rel.get("qualifier", ""),
            })
    relations_df = pd.DataFrame(relation_rows) if relation_rows else pd.DataFrame(
        columns=["ocel:eid", "ocel:oid", "ocel:qualifier"])

    # Build objects dataframe
    object_rows = []
    for obj in objects:
        object_rows.append({
            "ocel:oid": obj.get("id", ""),
            "ocel:type": obj.get("type", ""),
        })
    objects_df = pd.DataFrame(object_rows) if object_rows else pd.DataFrame(
        columns=["ocel:oid", "ocel:type"])

    ocel = OCEL(events=events_df, relations=relations_df, objects=objects_df)
    return ocel


def run_conformance(ocel2: dict, session_id: str) -> dict:
    """Run pm4py process discovery + conformance on an OCEL 2.0 log."""
    try:
        import pm4py
    except ImportError:
        print("ERROR: pm4py not installed. Run: pip install pm4py", file=sys.stderr)
        sys.exit(2)

    events = ocel2.get("events", [])
    activities = [e.get("type", "") for e in events]
    unique_activities = list(dict.fromkeys(activities))  # ordered unique

    print(f"[pm4py] Session: {session_id}")
    print(f"[pm4py] Events: {len(events)}")
    print(f"[pm4py] Activities (ordered): {activities}")

    # Convert to pm4py OCEL
    ocel = ocel2_to_pm4py(ocel2)

    # Discover process model using inductive miner
    try:
        process_tree = pm4py.discover_process_tree_inductive(ocel)
        net, im, fm = pm4py.convert_to_petri_net(process_tree)

        # Flatten to a standard event log for conformance checking
        flat_log = pm4py.ocel.flattening.flatten(ocel, unique_activities[0] if unique_activities else "GameSessionStarted")
        fitness_values = pm4py.fitness_token_based_replay(flat_log, net, im, fm)
        fitness = fitness_values.get("average_trace_fitness", 0.0)
        precision = pm4py.precision_token_based_replay(flat_log, net, im, fm)

    except Exception as e:
        print(f"[pm4py] Conformance check failed: {e}", file=sys.stderr)
        fitness = 0.0
        precision = 0.0

    # Verify the declared lifecycle is a substring of the actual trace
    declared_ok = all(act in activities for act in DECLARED_LIFECYCLE)
    declared_order_ok = False
    if declared_ok:
        try:
            positions = [activities.index(act) for act in DECLARED_LIFECYCLE]
            declared_order_ok = positions == sorted(positions)
        except ValueError:
            declared_order_ok = False

    verdict = "PASS" if (fitness >= FITNESS_THRESHOLD and declared_ok and declared_order_ok) else "FAIL"

    report = {
        "session_id": session_id,
        "verdict": verdict,
        "fitness": round(fitness, 4),
        "precision": round(precision, 4) if isinstance(precision, float) else None,
        "fitness_threshold": FITNESS_THRESHOLD,
        "declared_lifecycle": DECLARED_LIFECYCLE,
        "declared_ok": declared_ok,
        "declared_order_ok": declared_order_ok,
        "actual_activities": activities,
        "unique_activities": unique_activities,
        "total_events": len(events),
    }

    print(f"[pm4py] Fitness: {fitness:.4f} (threshold={FITNESS_THRESHOLD})")
    print(f"[pm4py] Precision: {precision}")
    print(f"[pm4py] Declared lifecycle ok: {declared_ok}, order ok: {declared_order_ok}")
    print(f"[pm4py] Verdict: {verdict}")

    return report


def main():
    parser = argparse.ArgumentParser(description="pm4py OCEL 2.0 conformance check")
    parser.add_argument("ocel_path", help="Path to OCEL 2.0 JSON file or - for stdin")
    parser.add_argument("--session-id", help="Override session_id in output (auto-detected otherwise)")
    parser.add_argument("--threshold", type=float, default=FITNESS_THRESHOLD,
                        help=f"Minimum fitness to pass (default {FITNESS_THRESHOLD})")
    parser.add_argument("--out", help="Output path for fitness_report.json (default: ocel/reports/)")
    args = parser.parse_args()

    global FITNESS_THRESHOLD
    FITNESS_THRESHOLD = args.threshold

    ocel2 = load_ocel2_json(args.ocel_path)

    # Auto-detect session_id from first event relationship or use override
    session_id = args.session_id
    if not session_id:
        events = ocel2.get("events", [])
        for evt in events:
            for rel in evt.get("relationships", []):
                if rel.get("qualifier") == "session":
                    session_id = rel.get("objectId", "unknown")
                    break
            if session_id:
                break
    if not session_id:
        session_id = "unknown"

    report = run_conformance(ocel2, session_id)

    # Write report
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    out_path = Path(args.out) if args.out else REPORTS_DIR / f"{session_id}_fitness_report.json"
    with open(out_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"[pm4py] Report written to {out_path}")

    # Exit code: 0 = PASS, 1 = FAIL
    sys.exit(0 if report["verdict"] == "PASS" else 1)


if __name__ == "__main__":
    main()
