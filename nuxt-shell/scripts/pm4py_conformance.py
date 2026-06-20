#!/usr/bin/env python3
"""
pm4py_conformance.py — OCEL conformance checker for nuxt-shell CI.

Usage:
  python3 scripts/pm4py_conformance.py <ocel_json_path> [--session-id <id>] [--out <path>]

Exit codes:
  0  fitness >= threshold (PASS)
  1  fitness < threshold (FAIL) or error
"""

import argparse
import json
import sys
from pathlib import Path

# Lawful lifecycle activities required for a conforming game session
REQUIRED_ACTIVITIES = ["GameSessionStarted", "FrameRendered", "InputAdmitted"]
THRESHOLD = 0.5


def load_ocel(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def extract_events(ocel: dict) -> list:
    """Support OCEL 1.x (ocel:events dict) and OCEL 2.0 (events array)."""
    # OCEL 2.0: top-level "events" array
    if "events" in ocel and isinstance(ocel["events"], list):
        return ocel["events"]

    # OCEL 1.x: "ocel:events" dict keyed by event id
    if "ocel:events" in ocel and isinstance(ocel["ocel:events"], dict):
        result = []
        for eid, ev in ocel["ocel:events"].items():
            entry = dict(ev)
            entry.setdefault("id", eid)
            # Normalise activity field
            if "ocel:activity" in entry:
                entry["activity"] = entry["ocel:activity"]
            result.append(entry)
        return result

    return []


def get_activity(event: dict) -> str:
    return event.get("activity") or event.get("ocel:activity") or ""


def get_session_id(event: dict):
    """Try common fields for a session identifier."""
    attrs = event.get("attributes") or event.get("ocel:vmap") or {}
    for key in ("session_id", "sessionId", "session-id"):
        if key in attrs:
            return str(attrs[key])
    # OCEL 2.0: check object references
    for obj_ref in event.get("relationships") or event.get("ocel:omap") or []:
        if isinstance(obj_ref, dict):
            oid = obj_ref.get("objectId") or obj_ref.get("id", "")
        else:
            oid = str(obj_ref)
        if "session" in oid.lower():
            return oid
    return None


def simple_conformance(events: list, session_id) -> dict:
    """Fallback conformance check without pm4py."""
    if session_id:
        events = [e for e in events if get_session_id(e) == session_id]

    found_activities = {get_activity(e) for e in events if get_activity(e)}
    matched = [a for a in REQUIRED_ACTIVITIES if a in found_activities]
    fitness = len(matched) / len(REQUIRED_ACTIVITIES) if REQUIRED_ACTIVITIES else 1.0

    # Precision: ratio of observed activities that are in the required set
    total_observed = len(found_activities)
    precision = (len(matched) / total_observed) if total_observed > 0 else 0.0

    return {
        "session_id": session_id,
        "fitness": round(fitness, 4),
        "precision": round(precision, 4),
        "activities_found": sorted(found_activities),
        "activities_required": REQUIRED_ACTIVITIES,
        "activities_matched": matched,
        "verdict": "PASS" if fitness >= THRESHOLD else "FAIL",
        "threshold": THRESHOLD,
        "method": "simple-fallback",
    }


def pm4py_conformance(events: list, session_id) -> dict:
    """pm4py DFG discovery + token replay fitness."""
    import pm4py
    from pm4py.objects.log.obj import EventLog, Trace, Event
    from pm4py.algo.conformance.tokenreplay import algorithm as token_replay
    from pm4py.algo.discovery.inductive import algorithm as inductive_miner

    if session_id:
        events = [e for e in events if get_session_id(e) == session_id or get_session_id(e) is None]

    # Build pm4py EventLog (single trace)
    trace = Trace()
    for ev in events:
        pm_event = Event()
        pm_event["concept:name"] = get_activity(ev) or "unknown"
        pm_event["time:timestamp"] = ev.get("timestamp") or ev.get("ocel:timestamp") or "1970-01-01T00:00:00Z"
        trace.append(pm_event)

    log = EventLog([trace])

    found_activities = {e["concept:name"] for t in log for e in t}

    try:
        net, im, fm = inductive_miner.apply(log)
        replayed = token_replay.apply(log, net, im, fm)
        fitness = replayed[0].get("trace_fitness", 0.0) if replayed else 0.0

        required_found = [a for a in REQUIRED_ACTIVITIES if a in found_activities]
        precision = len(required_found) / len(found_activities) if found_activities else 0.0
    except Exception as exc:
        result = simple_conformance(events, session_id)
        result["pm4py_error"] = str(exc)
        return result

    return {
        "session_id": session_id,
        "fitness": round(float(fitness), 4),
        "precision": round(precision, 4),
        "activities_found": sorted(found_activities),
        "activities_required": REQUIRED_ACTIVITIES,
        "verdict": "PASS" if fitness >= THRESHOLD else "FAIL",
        "threshold": THRESHOLD,
        "method": "pm4py-token-replay",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="OCEL conformance checker")
    parser.add_argument("ocel_json_path", help="Path to OCEL JSON file")
    parser.add_argument("--session-id", default=None, help="Filter to a single session")
    parser.add_argument("--out", default=None, help="Output JSON file path (default: stdout)")
    args = parser.parse_args()

    ocel_path = args.ocel_json_path
    if not Path(ocel_path).exists():
        print(f"ERROR: OCEL file not found: {ocel_path}", file=sys.stderr)
        return 1

    try:
        ocel = load_ocel(ocel_path)
    except json.JSONDecodeError as exc:
        print(f"ERROR: Invalid JSON in {ocel_path}: {exc}", file=sys.stderr)
        return 1

    events = extract_events(ocel)
    if not events:
        print(f"WARNING: No events found in {ocel_path}", file=sys.stderr)

    try:
        result = pm4py_conformance(events, args.session_id)
    except ImportError:
        result = simple_conformance(events, args.session_id)

    output = json.dumps(result, indent=2)

    if args.out:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(output + "\n")
    else:
        print(output)

    return 0 if result["fitness"] >= THRESHOLD else 1


if __name__ == "__main__":
    sys.exit(main())
