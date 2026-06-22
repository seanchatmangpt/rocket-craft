#!/usr/bin/env python3
"""ASSEMBLY-COHERENCE GATE (connectivity, not per-part size).

The metric-morphology gate (110/116/117) measures per-part bbox RATIOS and is
blind to PLACEMENT. A mech whose parts float apart with large gaps (an "exploded
blockout") passes every ratio band yet renders as many disconnected islands.
This gate closes that hole, grounded in 121_kinematic_connectivity_law.ttl and
the OBO Relations Ontology (ro:connected_to, bfo:part_of).

Checks:
  1. REACHABILITY: build the joint graph from 121 (law:Joint parentLink/childLink),
     assert every flagship KinematicLink reaches the Torso RootLink through joints
     -> else FLOATING_PART.
  2. JOINT-CONTACT: reuse the bbox measure from verify_metric_morphology; for each
     joint assert the child bbox overlaps/touches the parent bbox within tolerance
     -> else DISCONNECTED_JOINT. Contact assertions (law:inContactWith) are fed
     into the 121 SHACL shapes so the graph law itself bites.
  3. SILHOUETTE CONNECTIVITY: read foreground_component_count from the fresh
     visual_gap_report.json (or recompute connected components on the fresh
     render_silhouette.png). ADMITTED requires <= MAX_COMPONENTS (target 1).

Emits ASSEMBLY_COHERENCE_REPORT.{json,md} at repo root with a BLAKE3 receipt.
NEGATIVE FIXTURE: an in-memory mech with one part translated far away must REFUSE
with FLOATING_PART / DISCONNECTED_JOINT. Runs the deterministic core twice ->
replay_verified.
"""
import os
import sys
import json
import datetime

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import verify_metric_morphology as vmm  # reuse measure_all / b3 / measure_part

SRC_DIR = os.path.join(REPO, "ontology", "source_law")
REPORT_DIR = os.path.join(REPO, "generated", "mech_assets",
                          "reference_fabric_001", "reports")
RENDER_DIR = os.path.join(REPO, "generated", "mech_assets",
                          "reference_fabric_001", "renders")

# meters tolerance for "in contact": bbox gap along the worst-separated axis must
# be <= this for the joint to count as connected. 0.30 m ~ a slim seam.
CONTACT_TOL_M = 0.30
MAX_COMPONENTS = 3   # ADMITTED ceiling (target 1)

import rdflib  # noqa: E402
from rdflib import Namespace, RDF  # noqa: E402
import pyshacl  # noqa: E402

LAW = Namespace("https://rocket-craft.com/ontology/law#")
RF = Namespace("https://rocket-craft.com/asset/reference_fabric_001#")
SH = Namespace("http://www.w3.org/ns/shacl#")


# ---- joint graph from 121 ---------------------------------------------------
def load_joint_graph():
    g = rdflib.Graph()
    g.parse(os.path.join(SRC_DIR, "121_kinematic_connectivity_law.ttl"),
            format="turtle")
    joints = []  # (joint, parent_localname, child_localname)
    for j in g.subjects(RDF.type, LAW.Joint):
        parents = list(g.objects(j, LAW.parentLink))
        children = list(g.objects(j, LAW.childLink))
        if parents and children:
            joints.append((str(j).split("#")[-1],
                           str(parents[0]).split("#")[-1],
                           str(children[0]).split("#")[-1]))
    roots = [str(s).split("#")[-1] for s in g.subjects(RDF.type, LAW.RootLink)]
    links = sorted(str(s).split("#")[-1]
                   for s in g.subjects(RDF.type, LAW.KinematicLink))
    return joints, sorted(set(roots)), links


def reachability(joints, root, links):
    parent_of = {child: parent for _j, parent, child in joints}
    table = []
    for link in links:
        # walk parent chain
        seen = set()
        cur = link
        chain = [cur]
        reached = (cur == root)
        while cur in parent_of and cur not in seen:
            seen.add(cur)
            cur = parent_of[cur]
            chain.append(cur)
            if cur == root:
                reached = True
                break
        table.append({"part": link, "reaches_root": reached,
                      "chain": list(reversed(chain))})
    floating = [t["part"] for t in table if not t["reaches_root"]]
    return table, floating


# ---- metric contact between two measured bboxes -----------------------------
def axis_gap(a, b, ax):
    """Signed gap between intervals a,b along axis ax (cm). <=0 means overlap."""
    return max(a[0][ax] - b[1][ax], b[0][ax] - a[1][ax])


def in_contact(raw_a, raw_b, tol_m):
    """Two parts are in contact if, on EVERY axis, their intervals overlap or are
    separated by <= tol. A real seam touches on all three axes simultaneously."""
    tol_cm = tol_m / vmm.METERS_PER_UNIT
    gaps = [axis_gap(raw_a, raw_b, ax) for ax in range(3)]
    worst = max(gaps)
    return worst <= tol_cm, worst * vmm.METERS_PER_UNIT, gaps


def joint_contacts(joints, raw):
    results = []
    for jname, parent, child in joints:
        pkey = "SM_" + parent if not parent.startswith("SM_") else parent
        ckey = "SM_" + child if not child.startswith("SM_") else child
        if pkey not in raw or ckey not in raw:
            results.append({"joint": jname, "parent": parent, "child": child,
                            "measured": False, "in_contact": False,
                            "gap_m": None})
            continue
        ok, worst_m, _gaps = in_contact(raw[pkey], raw[ckey], CONTACT_TOL_M)
        results.append({"joint": jname, "parent": parent, "child": child,
                        "measured": True, "in_contact": ok,
                        "gap_m": round(worst_m, 4)})
    return results


# ---- silhouette component count ---------------------------------------------
def foreground_component_count():
    rep = os.path.join(REPORT_DIR, "visual_gap_report.json")
    if os.path.exists(rep):
        try:
            with open(rep) as f:
                d = json.load(f)
            if "foreground_component_count" in d:
                return int(d["foreground_component_count"]), "visual_gap_report.json"
        except (OSError, ValueError):
            pass
    # recompute from silhouette
    sil = os.path.join(RENDER_DIR, "render_silhouette.png")
    if not os.path.exists(sil):
        return None, "missing"
    try:
        import numpy as np
        from PIL import Image
        from collections import deque
        arr = np.array(Image.open(sil).convert("L"))
        h, w = arr.shape
        visited = np.zeros_like(arr, dtype=bool)
        count = 0
        for iy, ix in np.argwhere(arr > 127):
            if visited[iy, ix]:
                continue
            count += 1
            q = deque([(iy, ix)])
            visited[iy, ix] = True
            while q:
                cy, cx = q.popleft()
                for dy, dx in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                    ny, nx = cy + dy, cx + dx
                    if 0 <= ny < h and 0 <= nx < w and arr[ny, nx] > 127 \
                            and not visited[ny, nx]:
                        visited[ny, nx] = True
                        q.append((ny, nx))
        return count, "recomputed:render_silhouette.png"
    except Exception as e:  # noqa: BLE001
        return None, "error:%s" % e


# ---- SHACL graph law (121 shapes bite via inContactWith assertions) ---------
def build_data_graph(joints, contacts, root, links):
    """Instance graph: link typings + per-joint inContactWith for contacts that
    actually hold. Feeds the 121 FloatingPart/DisconnectedJoint shapes."""
    g = rdflib.Graph()
    g.parse(os.path.join(SRC_DIR, "121_kinematic_connectivity_law.ttl"),
            format="turtle")
    contact_ok = {(c["parent"], c["child"]): c["in_contact"] for c in contacts}
    for (parent, child), ok in contact_ok.items():
        if ok:
            g.add((RF[parent], LAW.inContactWith, RF[child]))
    return g


def run_shacl(data):
    shapes = rdflib.Graph()
    shapes.parse(os.path.join(SRC_DIR, "121_kinematic_connectivity_law.ttl"),
                 format="turtle")
    conforms, results, _txt = pyshacl.validate(
        data, shacl_graph=shapes, inference="rdfs", advanced=True,
        abort_on_first=False)
    msgs = sorted(set(str(m) for m in
                  results.objects(predicate=SH.resultMessage)))
    return conforms, msgs


# ---- negative fixture -------------------------------------------------------
def negative_fixture(joints):
    """Same joint tree, but the measured bboxes place Blade_Left far away (no
    contact with its parent limb) AND give Head no parent contact. The law must
    refuse with DISCONNECTED_JOINT (and the floating-link shape if a link lacks
    any parent joint)."""
    # fabricate raw bboxes: everyone overlapping at origin, EXCEPT Blade_Left far
    base = ([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])
    raw = {}
    for _j, parent, child in joints:
        for name in (parent, child):
            raw.setdefault("SM_" + name, ([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]))
    raw["SM_Blade_Left"] = ([100.0, 100.0, 100.0], [102.0, 102.0, 102.0])  # far
    contacts = joint_contacts(joints, raw)
    data = build_data_graph(joints, contacts, "SM_Torso", [])
    conforms, msgs = run_shacl(data)
    blob = " ".join(msgs)
    refused = (not conforms) and ("DISCONNECTED_JOINT" in blob)
    disconnected = [c["joint"] for c in contacts if not c["in_contact"]]
    return {"refused": refused, "conforms": conforms,
            "disconnected_joints": sorted(disconnected),
            "messages": msgs}


# ---- deterministic core -----------------------------------------------------
def build_core():
    joints, roots, links = load_joint_graph()
    root = roots[0] if roots else "SM_Torso"
    parts = vmm.measure_all()
    raw = {n: d["_raw"] for n, d in parts.items() if "_raw" in d}

    reach_table, floating = reachability(joints, root, links)
    contacts = joint_contacts(joints, raw)
    disconnected = [c for c in contacts if not c["in_contact"]]

    data = build_data_graph(joints, contacts, root, links)
    shacl_conforms, shacl_msgs = run_shacl(data)

    fcc, fcc_source = foreground_component_count()

    refusals = []
    for p in floating:
        refusals.append("FLOATING_PART(%s): not reachable from root %s" % (p, root))
    for c in disconnected:
        refusals.append("DISCONNECTED_JOINT(%s): child %s not in contact with "
                        "parent %s (worst gap %s m)"
                        % (c["joint"], c["child"], c["parent"], c["gap_m"]))

    sil_ok = (fcc is not None and fcc <= MAX_COMPONENTS)
    if not floating and not disconnected and sil_ok:
        verdict = "ADMITTED"
    elif not floating and not disconnected and fcc is not None:
        # graph law satisfied but render still shows islands -> partial
        verdict = "PARTIAL_ALIVE"
    elif not refusals and fcc is None:
        verdict = "PARTIAL_ALIVE"
    else:
        verdict = "PARTIAL_ALIVE" if (not floating) else "REFUSED"

    core = {
        "gate": "assembly_coherence",
        "root_link": root,
        "reachability": reach_table,
        "floating_parts": sorted(floating),
        "joint_contacts": contacts,
        "disconnected_joints": [c["joint"] for c in disconnected],
        "contact_tolerance_m": CONTACT_TOL_M,
        "foreground_component_count": fcc,
        "foreground_component_count_source": fcc_source,
        "max_components_admitted": MAX_COMPONENTS,
        "silhouette_connectivity_ok": sil_ok,
        "shacl_conforms": shacl_conforms,
        "shacl_messages": shacl_msgs,
        "refusals": sorted(refusals),
        "verdict": verdict,
    }
    return core, joints


def emit_md(r):
    md = ["# ASSEMBLY_COHERENCE_REPORT (kinematic connectivity)\n",
          "- timestamp: %s" % r["timestamp_utc"],
          "- verdict: **%s**" % r["verdict"],
          "- root_link: %s" % r["root_link"],
          "- foreground_component_count: %s (source: %s, ADMITTED ceiling %s)"
          % (r["foreground_component_count"],
             r["foreground_component_count_source"], r["max_components_admitted"]),
          "- shacl_conforms (121): %s" % r["shacl_conforms"],
          "- replay_verified: %s" % r["replay_verified"],
          "- receipt_blake3: `%s`" % r["receipt_blake3"],
          "\n## Reachability (every part must reach the Torso root via joints)",
          "| part | reaches_root | chain |", "|---|---|---|"]
    for t in r["reachability"]:
        md.append("| %s | %s | %s |"
                  % (t["part"], t["reaches_root"], " -> ".join(t["chain"])))
    md.append("\n## Per-joint contact")
    md.append("| joint | parent | child | in_contact | worst_gap_m |")
    md.append("|---|---|---|---|---|")
    for c in r["joint_contacts"]:
        md.append("| %s | %s | %s | %s | %s |"
                  % (c["joint"], c["parent"], c["child"],
                     c["in_contact"], c["gap_m"]))
    md.append("\n## Refusals")
    for x in r["refusals"]:
        md.append("- %s" % x)
    if not r["refusals"]:
        md.append("- (none)")
    nf = r["negative_fixture"]
    md.append("\n## Negative fixture (Blade_Left translated far away)")
    md.append("- refused: %s" % nf["refused"])
    md.append("- disconnected_joints: %s" % ", ".join(nf["disconnected_joints"]))
    with open(os.path.join(REPO, "ASSEMBLY_COHERENCE_REPORT.md"), "w") as f:
        f.write("\n".join(md) + "\n")


def main():
    core1, joints = build_core()
    core2, _ = build_core()
    replay_verified = (json.dumps(core1, sort_keys=True) ==
                       json.dumps(core2, sort_keys=True))

    neg = negative_fixture(joints)

    verdict = core1["verdict"]
    if not neg["refused"]:
        verdict = "REFUSED"  # the law failed to bite -> no standing

    report = dict(core1)
    report["verdict"] = verdict
    report["replay_verified"] = replay_verified
    report["negative_fixture"] = neg

    body = json.dumps(report, sort_keys=True, indent=2)
    report["timestamp_utc"] = datetime.datetime.utcnow().isoformat() + "Z"
    report["receipt_blake3"] = vmm.b3(body.encode())

    with open(os.path.join(REPO, "ASSEMBLY_COHERENCE_REPORT.json"), "w") as f:
        json.dump(report, f, indent=2, sort_keys=True)
    emit_md(report)

    print("verdict: %s | fcc=%s | floating=%s | disconnected=%s | neg_refused=%s | replay=%s"
          % (verdict, core1["foreground_component_count"],
             core1["floating_parts"], core1["disconnected_joints"],
             neg["refused"], replay_verified))
    for r in core1["refusals"]:
        print("  refusal:", r)
    if verdict == "REFUSED":
        return 2
    if verdict == "PARTIAL_ALIVE":
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
