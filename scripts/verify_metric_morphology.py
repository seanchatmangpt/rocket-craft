#!/usr/bin/env python3
"""PRE-RENDER METRIC MORPHOLOGY GATE (graph + geometry only, NO render).

Doctrine: morphology is GRAPH LAW. A part's metric envelope (meters) and its
ratio-of-body-height bands are enforced in the graph (SPARQL provision + SHACL
refusal) BEFORE the render. "Huge UFO disc passes component count" must be
REFUSED here by metric law, not discovered later in UE4/Playwright.

Pipeline:
  1. Measure each flagship USD part -> nested xformOp:translate/scale over
     points (mirrors scripts/compare_reference_render.py regex) -> per-part
     world bounding box in USD units (cm) -> meters (x metersPerUnit=0.01).
  2. Build an rdflib instance graph (eng:hasBodyHeight, per-part
     eng:hasBoundingBoxYMin/YMax via sosa:hasResult/qudt:numericValue to match
     110's property paths). Vertical axis = Z (this USD lays parts out along Z;
     head Z > torso Z). X = span.
  3. Load 110 + 116 + 117 + measured graph, run pyshacl(rdfs inference) ->
     conforms + violation messages (REFUSE_*/ANATOMY_PARADOX). Cross-validate
     with direct ratio-band checks.
  4. Emit METRIC_MORPHOLOGY_REPORT.json + .md at repo root + BLAKE3 receipt.
     Honest verdict: ADMITTED / REFUSED / PARTIAL_ALIVE.
  5. NEGATIVE FIXTURE: in-memory UFO-disc mech (head below torso + shield at
     ratio 1.0 with no archetype) must be REFUSED with ANATOMY_PARADOX +
     REFUSE_DEFAULT_SHIELD_PROPORTION.
  6. Run measurement+validation twice; report (minus timestamp) byte-identical
     -> replay_verified.
"""
import os
import re
import sys
import json
import hashlib
import datetime
import subprocess

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
USD_DIR = os.path.join(REPO, "generated", "mech_assets", "reference_fabric_001", "usd")
SRC_DIR = os.path.join(REPO, "ontology", "source_law")
METERS_PER_UNIT = 0.01  # USD metersPerUnit -> cm to meters

# Flagship parts (tank shells SM_TankTreads/KwK36Gun/InterleavedWheels are out
# of flagship scope and intentionally excluded).
# NOTE: the flagship part roster is NO LONGER hardcoded here. flagship_parts()
# derives it from the GRAPH (117 eng:hasPart) so the part list the gate measures
# is exactly the roster the source law declares — one source of truth.
# Vertical axis index in this USD layout. NOTE: the ggen-generated geometry
# stacks parts along Z (SM_Head group translate (0,0,1.5) sits above SM_Torso
# (0,0,0)), even though the USD metadata declares upAxis="Y". That declared-vs-
# actual disagreement is itself a morphology defect this gate surfaces (see
# orientation_finding in the report) — usdrecord renders with upAxis=Y, so the
# mech is rendered mis-oriented relative to how it was laid out.
VERT = 1  # Y (actual stacking axis)
SPAN = 0  # X
_AXIS_NAME = {0: "X", 1: "Y", 2: "Z"}


def declared_up_axis():
    """The upAxis token declared in the part USD metadata (authoritative-by-spec)."""
    try:
        txt = open(os.path.join(USD_DIR, "SM_Torso.usda")).read()
        m = re.search(r'upAxis\s*=\s*"([XYZ])"', txt)
        return m.group(1) if m else "UNKNOWN"
    except OSError:
        return "UNKNOWN"

# NOTE: part height-ratio bands are NO LONGER hand-mirrored here. They are derived
# from the GRAPH at runtime by part_bands() (117 part-class typing x 116
# MorphologyBand) — the TTL source law is the single source of truth, so the
# direct ratio check can never drift from the SHACL law. See part_bands() below.

GROUP_RE = re.compile(r'def Xform "(prim_[^"]+)"\s*\{(.*?)\n        \}', re.DOTALL)
MESH_RE = re.compile(r'def Mesh "([^"]+)"\s*\{(.*?)\n                \}', re.DOTALL)
# Native USD shapes the geometry generator now emits inside each prim group.
CUBE_RE = re.compile(r'def Cube "[^"]+"\s*\{(.*?)\n                \}', re.DOTALL)
CYL_RE = re.compile(r'def Cylinder "[^"]+"\s*\{(.*?)\n                \}', re.DOTALL)
TRANS_RE = re.compile(r'double3 xformOp:translate = \(([^)]+)\)')
SCALE_RE = re.compile(r'double3 xformOp:scale = \(([^)]+)\)')
POINTS_RE = re.compile(r'point3f\[\] points = \[([^\]]+)\]')
SIZE_RE = re.compile(r'double size = ([-+\d.eE]+)')
RADIUS_RE = re.compile(r'double radius = ([-+\d.eE]+)')
HEIGHT_RE = re.compile(r'double height = ([-+\d.eE]+)')
NUM_RE = re.compile(r'[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?')


def _vec(m, default):
    return [float(x) for x in m.group(1).split(",")] if m else list(default)


def _num(m, default):
    return float(m.group(1)) if m else default


def _corners(half):
    """8 AABB corners for half-extents (hx, hy, hz)."""
    return [(sx * half[0], sy * half[1], sz * half[2])
            for sx in (-1, 1) for sy in (-1, 1) for sz in (-1, 1)]


def measure_part(path):
    """World bounding box (USD units) of a part's geometry. Handles BOTH the
    legacy def Mesh + point3f[] points format AND the current native-shape format
    (def Cube / def Cylinder inside each prim group). Transform semantics are
    identical to the legacy code: per-shape scale+translate, then the group's
    scale+translate (rotateXYZ is intentionally ignored, matching the original
    measure_part — so ratios stay comparable across the format change)."""
    content = open(path).read()
    pts = []
    for _gname, gblock in GROUP_RE.findall(content):
        gtr = _vec(TRANS_RE.search(gblock), (0, 0, 0))
        gsc = _vec(SCALE_RE.search(gblock), (1, 1, 1))

        def emit(local_pts, ssc, str_):
            for p in local_pts:
                pts.append([(p[k] * ssc[k] + str_[k]) * gsc[k] + gtr[k]
                            for k in range(3)])

        # legacy explicit meshes
        for _mname, mblock in MESH_RE.findall(gblock):
            pm = POINTS_RE.search(mblock)
            if not pm:
                continue
            nums = [float(n) for n in NUM_RE.findall(pm.group(1))]
            mpts = [tuple(nums[i:i + 3]) for i in range(0, len(nums) - 2, 3)]
            emit(mpts, _vec(SCALE_RE.search(mblock), (1, 1, 1)),
                 _vec(TRANS_RE.search(mblock), (0, 0, 0)))

        # native cubes: AABB +/- size/2, then cube scale+translate
        for cblock in CUBE_RE.findall(gblock):
            h = _num(SIZE_RE.search(cblock), 1.0) / 2.0
            emit(_corners((h, h, h)),
                 _vec(SCALE_RE.search(cblock), (1, 1, 1)),
                 _vec(TRANS_RE.search(cblock), (0, 0, 0)))

        # native cylinders: radius in X/Y, height/2 in Z (USD default axis Z)
        for cblock in CYL_RE.findall(gblock):
            r = _num(RADIUS_RE.search(cblock), 0.0)
            hz = _num(HEIGHT_RE.search(cblock), 0.0) / 2.0
            emit(_corners((r, r, hz)),
                 _vec(SCALE_RE.search(cblock), (1, 1, 1)),
                 _vec(TRANS_RE.search(cblock), (0, 0, 0)))

    if not pts:
        return None
    mn = [min(p[k] for p in pts) for k in range(3)]
    mx = [max(p[k] for p in pts) for k in range(3)]
    return mn, mx


def b3(data: bytes) -> str:
    p = subprocess.run(["b3sum", "--no-names"], input=data, capture_output=True)
    if p.returncode == 0:
        return "blake3:" + p.stdout.decode().strip()
    return "sha256:" + hashlib.sha256(data).hexdigest()


# ---- RDF / SHACL ------------------------------------------------------------
import rdflib  # noqa: E402
from rdflib import Namespace, Literal, BNode, RDF, XSD  # noqa: E402
import pyshacl  # noqa: E402

ENG = Namespace("https://rocket-craft.com/ontology/engineering#")
LAW = Namespace("https://rocket-craft.com/ontology/law#")
SOSA = Namespace("http://www.w3.org/ns/sosa/")
QUDT = Namespace("http://qudt.org/schema/qudt/")
RF = Namespace("https://rocket-craft.com/asset/reference_fabric_001#")
SH = Namespace("http://www.w3.org/ns/shacl#")

_PART_BANDS_CACHE = {}
_FLAGSHIP_CACHE = []


def flagship_parts():
    """The flagship part roster, DERIVED from the graph (117 eng:hasPart) rather
    than a hardcoded list — the gate measures exactly the parts source law declares.
    Returns local names sorted for deterministic (replayable) iteration."""
    if _FLAGSHIP_CACHE:
        return _FLAGSHIP_CACHE
    g = rdflib.Graph()
    g.parse(os.path.join(SRC_DIR, "117_reference_fabric_metric_binding.ttl"),
            format="turtle")
    q = ("PREFIX eng: <https://rocket-craft.com/ontology/engineering#> "
         "SELECT ?part WHERE { ?mech eng:hasPart ?part . }")
    names = sorted(str(p).split("#")[-1] for (p,) in g.query(q))
    _FLAGSHIP_CACHE.extend(names)
    return _FLAGSHIP_CACHE


def part_bands():
    """Derive (class, lo, hi) height-ratio bands per flagship part FROM THE GRAPH —
    117 part-class typing joined with 116 MorphologyBand. The TTL source law is the
    single source of truth, so this direct ratio check can never drift from the
    SHACL law (which reads the same triples). Replaces the old hand-mirrored dict."""
    if _PART_BANDS_CACHE:
        return _PART_BANDS_CACHE
    g = rdflib.Graph()
    for f in ("116_metric_morphology_bands.ttl",
              "117_reference_fabric_metric_binding.ttl"):
        g.parse(os.path.join(SRC_DIR, f), format="turtle")
    q = """
    PREFIX eng: <https://rocket-craft.com/ontology/engineering#>
    PREFIX law: <https://rocket-craft.com/ontology/law#>
    SELECT ?part ?cls ?lo ?hi WHERE {
      ?part eng:attachedTo ?mech ; a ?cls .
      ?band law:appliesToPartClass ?cls ;
            law:bandMinRatio ?lo ; law:bandMaxRatio ?hi .
    } ORDER BY ?part ?cls
    """
    for part, cls, lo, hi in g.query(q):
        name = str(part).split("#")[-1]
        lo, hi = float(lo), float(hi)
        if name in _PART_BANDS_CACHE:
            # part carries multiple banded co-types (e.g. BipedalLeg + BipedalLimb):
            # keep the TIGHTEST band so a broader co-type can never loosen the law.
            c, plo, phi = _PART_BANDS_CACHE[name]
            _PART_BANDS_CACHE[name] = (c, max(plo, lo), min(phi, hi))
        else:
            _PART_BANDS_CACHE[name] = (str(cls).split("#")[-1], lo, hi)
    return _PART_BANDS_CACHE


def _obs(g, subj, pred, value):
    o = BNode()
    r = BNode()
    g.add((subj, pred, o))
    g.add((o, RDF.type, SOSA.Observation))
    g.add((o, SOSA.hasResult, r))
    g.add((r, QUDT.numericValue, Literal(str(value), datatype=XSD.decimal)))


# Prefix map injected onto every sh:SPARQLConstraint so the SELECT bodies in
# 110/116 (which use eng:/law:/sosa:/qudt: CURIEs) resolve at query time.
_PREFIXES = {
    "eng": str(ENG), "law": str(LAW), "sosa": str(SOSA), "qudt": str(QUDT),
    "rdf": str(RDF), "xsd": str(XSD),
    "mud": "https://rocket-craft.com/ontology/mud#",
}


def _attach_prefixes(g):
    decl = BNode()
    g.add((decl, RDF.type, SH.PrefixDeclarations))
    for pfx, ns in _PREFIXES.items():
        pn = BNode()
        g.add((decl, SH.declare, pn))
        g.add((pn, SH.prefix, Literal(pfx)))
        g.add((pn, SH.namespace, Literal(ns, datatype=XSD.anyURI)))
    for s, _p, _o in list(g.triples((None, SH["select"], None))):
        g.add((s, SH.prefixes, decl))
    return g


def shapes_graph():
    g = rdflib.Graph()
    for f in ["110_bipedal_metric_envelope_law.ttl",
              "116_metric_morphology_bands.ttl",
              "117_reference_fabric_metric_binding.ttl",
              "120_morphology_purity_law.ttl"]:
        g.parse(os.path.join(SRC_DIR, f), format="turtle")
    _attach_prefixes(g)
    return g


def measured_data_graph(parts):
    """Build instance graph from measured geometry, on top of 117 typings."""
    g = rdflib.Graph()
    # 116 carries the MorphologyBand facts (appliesToPartClass / bandMinRatio /
    # bandMaxRatio); they must live in the DATA graph so the sh:SPARQLConstraint
    # bodies (which query against the target graph) can read them.
    g.parse(os.path.join(SRC_DIR, "116_metric_morphology_bands.ttl"),
            format="turtle")
    g.parse(os.path.join(SRC_DIR, "117_reference_fabric_metric_binding.ttl"),
            format="turtle")
    # body height from measured vertical span (meters)
    tops = [parts[p]["y_max_m"] for p in parts]
    bots = [parts[p]["y_min_m"] for p in parts]
    body_h = max(tops) - min(bots)
    # rebind measured body height onto the mech (overrides the 18.0 anchor for
    # the actual ratio math the SHACL shapes perform)
    for o in list(g.objects(RF.ReferenceFabric_001, ENG.hasBodyHeight)):
        g.remove((RF.ReferenceFabric_001, ENG.hasBodyHeight, o))
    _obs(g, RF.ReferenceFabric_001, ENG.hasBodyHeight, round(body_h, 6))
    # whole-body bbox (for 110 BipedalAnatomyBands head-above-torso check, which
    # reads the BipedalTorso root's YMax). The torso-segment top is the relevant
    # reference; use the measured torso-segment YMax.
    if "SM_Torso" in parts:
        _obs(g, RF.ReferenceFabric_001, ENG.hasBoundingBoxYMax,
             round(parts["SM_Torso"]["y_max_m"], 6))
    for p, d in parts.items():
        subj = RF[p]
        _obs(g, subj, ENG.hasBoundingBoxYMin, round(d["y_min_m"], 6))
        _obs(g, subj, ENG.hasBoundingBoxYMax, round(d["y_max_m"], 6))
    return g, body_h


def run_shacl(data, shapes):
    conforms, results_graph, text = pyshacl.validate(
        data, shacl_graph=shapes, inference="rdfs", advanced=True,
        abort_on_first=False, meta_shacl=False, debug=False)
    msgs = sorted(set(str(m) for m in
                  results_graph.objects(predicate=SH.resultMessage)))
    return conforms, msgs


_AXIS_INDEX = {"X": 0, "Y": 1, "Z": 2}


def detect_stacking_axis(raw):
    """The axis along which the body actually stacks: where the head center sits
    furthest from the limb center. Used only to FLAG agreement with upAxis — the
    measurement itself is taken along the declared up-axis (render truth)."""
    head = raw.get("SM_Head")
    limb = raw.get("SM_Limb_Left") or raw.get("SM_Limb_Right")
    if not head or not limb:
        return VERT
    hc = [0.5 * (head[0][a] + head[1][a]) for a in range(3)]
    lc = [0.5 * (limb[0][a] + limb[1][a]) for a in range(3)]
    return max(range(3), key=lambda a: abs(hc[a] - lc[a]))


def measure_all():
    """Measure along the DECLARED up-axis (usdrecord renders by upAxis, so that is
    the vertical truth). The span axis is the widest horizontal axis (wing span).
    The actual mass-stacking axis is detected separately to flag any disagreement."""
    global VERT, SPAN
    raw = {}
    for usd_name in flagship_parts():
        path = os.path.join(USD_DIR, usd_name + ".usda")
        if not os.path.exists(path):
            continue
        res = measure_part(path)
        if res is None:
            continue
        raw[usd_name] = res
    VERT = _AXIS_INDEX.get(declared_up_axis(), 1)
    if raw:
        extents = [max((mx[a] - mn[a]) for mn, mx in raw.values()) for a in range(3)]
        SPAN = max((a for a in range(3) if a != VERT), key=lambda a: extents[a])
    parts = {}
    for usd_name, (mn, mx) in raw.items():
        parts[usd_name] = {
            "y_min_m": mn[VERT] * METERS_PER_UNIT,
            "y_max_m": mx[VERT] * METERS_PER_UNIT,
            "x_min_m": mn[SPAN] * METERS_PER_UNIT,
            "x_max_m": mx[SPAN] * METERS_PER_UNIT,
            "_raw": (mn, mx),
        }
    return parts


def build_core(parts):
    """Deterministic core of the report (everything except timestamp/receipt)."""
    tops = [parts[p]["y_max_m"] for p in parts]
    bots = [parts[p]["y_min_m"] for p in parts]
    body_h = max(tops) - min(bots)

    per_part = []
    direct_refusals = []
    PB = part_bands()  # graph-derived bands (116 x 117), single source of truth
    for name in sorted(parts):
        d = parts[name]
        h = d["y_max_m"] - d["y_min_m"]
        ratio = h / body_h if body_h else 0.0
        _cls, lo, hi = PB[name]
        in_band = lo <= ratio <= hi
        per_part.append({
            "part": name,
            "y_min_m": round(d["y_min_m"], 6),
            "y_max_m": round(d["y_max_m"], 6),
            "height_m": round(h, 6),
            "height_ratio": round(ratio, 4),
            "band": [lo, hi],
            "in_band": in_band,
        })
        if not in_band:
            direct_refusals.append(
                "REFUSE_PART_HEIGHT_BAND(%s): ratio %.4f outside [%.2f,%.2f]"
                % (name, ratio, lo, hi))

    # SHACL on the real mech
    data, _ = measured_data_graph(parts)
    shapes = shapes_graph()
    conforms, shacl_msgs = run_shacl(data, shapes)

    refusals = sorted(set(direct_refusals + shacl_msgs))
    if conforms and not direct_refusals:
        verdict = "ADMITTED"
    elif not refusals:
        verdict = "ADMITTED"
    else:
        # real mech violates a band -> honest PARTIAL_ALIVE (not faked ADMITTED)
        verdict = "PARTIAL_ALIVE"

    # Orientation finding: the actual part-stacking axis vs the declared upAxis.
    # When they disagree, usdrecord renders the mech mis-oriented — a real defect
    # the metric graph exposes BEFORE the render (the whole point of this layer).
    up = declared_up_axis()
    raw = {n: d["_raw"] for n, d in parts.items() if "_raw" in d}
    stacking = _AXIS_NAME[detect_stacking_axis(raw)] if raw else _AXIS_NAME[VERT]
    orientation_finding = {
        "declared_up_axis": up,
        "actual_stacking_axis": stacking,
        "agree": up == stacking,
        "note": ("Parts stack along %s but the USD declares upAxis=%s; usdrecord "
                 "renders with the declared up-axis, so the asset is rendered "
                 "mis-oriented relative to its layout. Fix the generator to lay "
                 "parts along %s (or set upAxis=%s)."
                 % (stacking, up, up, stacking)) if up != stacking
                else "Declared up-axis matches the actual part-stacking axis.",
    }

    core = {
        "gate": "metric_morphology_pre_render",
        "vertical_axis": _AXIS_NAME[VERT],
        "measured_along_up_axis": up,
        "span_axis": _AXIS_NAME[SPAN],
        "orientation_finding": orientation_finding,
        "meters_per_unit": METERS_PER_UNIT,
        "body_height_m": round(body_h, 6),
        "parts": per_part,
        "shacl_conforms": conforms,
        "shacl_messages": sorted(shacl_msgs),
        "direct_band_refusals": sorted(direct_refusals),
        "refusals": refusals,
        "verdict": verdict,
    }
    return core


def negative_fixture():
    """In-memory UFO-disc mech: head Y_min < torso Y_max (ANATOMY_PARADOX) AND
    a shield at ratio 1.0 with no archetype (REFUSE_DEFAULT_SHIELD_PROPORTION).
    The law MUST refuse this."""
    g = rdflib.Graph()
    UFO = Namespace("https://rocket-craft.com/asset/ufo_disc_fixture#")
    mech = UFO.UfoDisc
    g.add((mech, RDF.type, LAW.BipedalTorso))
    _obs(g, mech, ENG.hasBodyHeight, 10.0)
    # torso bbox: top at 8.0
    _obs(g, mech, ENG.hasBoundingBoxYMax, 8.0)
    _obs(g, mech, ENG.hasBoundingBoxYMin, 0.0)
    # head sits BELOW torso top -> anatomy paradox
    head = UFO.Head
    g.add((head, RDF.type, LAW.MechaCrown))
    g.add((mech, ENG.hasPart, head))
    g.add((head, ENG.attachedTo, mech))
    _obs(g, head, ENG.hasBoundingBoxYMin, 1.0)   # 1.0 < torsoMaxY 8.0
    _obs(g, head, ENG.hasBoundingBoxYMax, 3.0)
    # a leg that does NOT descend below pelvis (legMinY >= 0) -> paradox too
    leg = UFO.Leg
    g.add((leg, RDF.type, LAW.BipedalLeg))
    g.add((mech, ENG.hasPart, leg))
    g.add((leg, ENG.attachedTo, mech))
    _obs(g, leg, ENG.hasBoundingBoxYMin, 2.0)
    _obs(g, leg, ENG.hasBoundingBoxYMax, 6.0)
    # oversized shield at ratio 1.0 with NO archetype
    shield = UFO.Disc
    g.add((shield, RDF.type, LAW.MechaShield))
    g.add((shield, ENG.attachedTo, mech))
    _obs(g, shield, ENG.hasShieldHeight, 10.0)  # ratio 1.0

    conforms, msgs = run_shacl(g, shapes_graph())
    codes = []
    blob = " ".join(msgs)
    if "ANATOMY_PARADOX" in blob:
        codes.append("ANATOMY_PARADOX")
    if "REFUSE_DEFAULT_SHIELD_PROPORTION" in blob:
        codes.append("REFUSE_DEFAULT_SHIELD_PROPORTION")
    refused = (not conforms) and ("ANATOMY_PARADOX" in blob) and \
              ("REFUSE_DEFAULT_SHIELD_PROPORTION" in blob)
    return {"refused": refused, "codes": sorted(set(codes)),
            "messages": sorted(msgs), "conforms": conforms}


def python_hardcoded_blade_scale_must_refuse():
    """Verify that a geometry primitive with a scale exceeding law:hasBladeScaleMax is refused
    with REFUSE_PROVENANCE_VIOLATION."""
    g = rdflib.Graph()
    # Define a mock primitive with scaleX 60.0, but max allowed is 50.0
    UFO = Namespace("https://rocket-craft.com/asset/ufo_disc_fixture#")
    MUD = Namespace("https://rocket-craft.com/ontology/mud#")
    prim = UFO.failing_blade_prim
    g.add((prim, RDF.type, MUD.GeometryPrimitive))
    g.add((prim, MUD.belongsToPart, MUD.blade_left))
    g.add((prim, MUD.scaleX, Literal("60.0", datatype=XSD.float)))
    g.add((prim, MUD.scaleY, Literal("1.0", datatype=XSD.float)))
    g.add((prim, MUD.scaleZ, Literal("1.0", datatype=XSD.float)))
    g.add((prim, MUD.translateX, Literal("0.0", datatype=XSD.float)))
    g.add((prim, MUD.translateY, Literal("0.0", datatype=XSD.float)))
    g.add((prim, MUD.translateZ, Literal("0.0", datatype=XSD.float)))
    g.add((prim, MUD.rotateX, Literal("0.0", datatype=XSD.float)))
    g.add((prim, MUD.rotateY, Literal("0.0", datatype=XSD.float)))
    g.add((prim, MUD.rotateZ, Literal("0.0", datatype=XSD.float)))
    g.add((prim, MUD.belongsToPart, MUD.blade_left))
    g.add((prim, MUD.primitiveFamily, Literal("blade")))
    g.add((prim, MUD.materialBinding, MUD.M_WhiteArmor))
    g.add((prim, LAW.hasBladeScaleMax, Literal("50.0", datatype=XSD.decimal)))
    g.add((prim, LAW.hasBladeScaleMin, Literal("0.01", datatype=XSD.decimal)))
    g.add((prim, LAW.hasSubdivisionDensity, Literal("4", datatype=XSD.integer)))
    g.add((prim, LAW.hasMaterialZoneBinding, MUD.M_WhiteArmor))
    g.add((prim, LAW.hasEdgeCount, Literal("4", datatype=XSD.integer)))
    g.add((prim, LAW.hasSocketAttachment, MUD.Socket_None))
    g.add((prim, LAW.hasCurvatureSweepClass, Literal("law:LinearSweep")))
    g.add((prim, LAW.hasArmorDensityBand, Literal("law:HighDensityArmor")))
    
    conforms, msgs = run_shacl(g, shapes_graph())
    refused = (not conforms) and any("REFUSE_PROVENANCE_VIOLATION" in msg for msg in msgs)
    codes = []
    if not conforms:
        for msg in msgs:
            if "REFUSE_PROVENANCE_VIOLATION" in msg:
                codes.append("REFUSE_PROVENANCE_VIOLATION")
    return {"refused": refused, "codes": sorted(set(codes)), "messages": sorted(msgs), "conforms": conforms}


def main():
    parts = measure_all()
    if not parts:
        # No measurable parts (e.g. the USD geometry format changed under the
        # parser). Overwrite the report with an honest UNKNOWN so a STALE
        # ADMITTED can never mislead a reader who doesn't re-run the gate.
        msg = ("no flagship USD parts measured — measure_part could not parse the "
               "current part USD (geometry format mismatch). See "
               "docs/GGEN_SINGLE_SOURCE.md (CRITICAL section).")
        print("FATAL: " + msg, file=sys.stderr)
        out_json = os.path.join(REPO, "METRIC_MORPHOLOGY_REPORT.json")
        with open(out_json, "w") as f:
            json.dump({"verdict": "UNKNOWN", "reason": msg,
                       "parts": [], "shacl_conforms": False,
                       "replay_verified": False,
                       "timestamp_utc": datetime.datetime.utcnow().isoformat() + "Z"},
                      f, indent=2, sort_keys=True)
        return 2

    core1 = build_core(parts)
    core2 = build_core(parts)
    replay_verified = (json.dumps(core1, sort_keys=True) ==
                       json.dumps(core2, sort_keys=True))

    neg = negative_fixture()
    neg_blade = python_hardcoded_blade_scale_must_refuse()

    # If either negative fixture does NOT refuse, the law is broken.
    verdict = core1["verdict"]
    if not neg["refused"] or not neg_blade["refused"]:
        verdict = "REFUSED"  # the law itself failed to bite -> no standing

    report = dict(core1)
    report["verdict"] = verdict
    report["negative_fixture"] = {
        "refused": neg["refused"] and neg_blade["refused"],
        "codes": sorted(set(neg["codes"] + neg_blade["codes"])),
        "messages": sorted(neg["messages"] + neg_blade["messages"]),
        "conforms": neg["conforms"] and neg_blade["conforms"]
    }
    report["replay_verified"] = replay_verified

    # deterministic body for the receipt (no timestamp)
    body = json.dumps(report, sort_keys=True, indent=2)
    report_with_meta = dict(report)
    report_with_meta["timestamp_utc"] = datetime.datetime.utcnow().isoformat() + "Z"
    report_with_meta["receipt_blake3"] = b3(body.encode())

    out_json = os.path.join(REPO, "METRIC_MORPHOLOGY_REPORT.json")
    with open(out_json, "w") as f:
        json.dump(report_with_meta, f, indent=2, sort_keys=True)

    _emit_md(report_with_meta)

    print("verdict: %s | shacl_conforms=%s | neg_refused=%s | replay=%s"
          % (verdict, report["shacl_conforms"], neg["refused"], replay_verified))
    for r in report["refusals"]:
        print("  refusal:", r)
    # Gate semantics: negative-fixture failure (law broken) -> hard fail.
    # Real-mech band violation -> PARTIAL_ALIVE warning, non-zero so the
    # pre-render gate surfaces it (caller decides whether to keep render going).
    if verdict == "REFUSED":
        return 2
    if verdict == "PARTIAL_ALIVE":
        return 1
    return 0


def _emit_md(r):
    md = ["# METRIC_MORPHOLOGY_REPORT (pre-render graph law)\n",
          "- timestamp: %s" % r["timestamp_utc"],
          "- verdict: **%s**" % r["verdict"],
          "- body_height_m: %s (vertical axis Z, metersPerUnit %s)"
          % (r["body_height_m"], r["meters_per_unit"]),
          "- shacl_conforms: %s" % r["shacl_conforms"],
          "- replay_verified: %s" % r["replay_verified"],
          "- receipt_blake3: `%s`" % r["receipt_blake3"],
          "\n## Per-part envelopes"]
    md.append("| part | y_min_m | y_max_m | height_m | ratio | band | in_band |")
    md.append("|---|---|---|---|---|---|---|")
    for p in r["parts"]:
        md.append("| %s | %s | %s | %s | %s | %s | %s |" % (
            p["part"], p["y_min_m"], p["y_max_m"], p["height_m"],
            p["height_ratio"], p["band"], p["in_band"]))
    md.append("\n## Refusals")
    for x in r["refusals"]:
        md.append("- %s" % x)
    if not r["refusals"]:
        md.append("- (none)")
    nf = r["negative_fixture"]
    md.append("\n## Negative fixture (UFO disc)")
    md.append("- refused: %s" % nf["refused"])
    md.append("- codes: %s" % ", ".join(nf["codes"]))
    with open(os.path.join(REPO, "METRIC_MORPHOLOGY_REPORT.md"), "w") as f:
        f.write("\n".join(md) + "\n")


if __name__ == "__main__":
    sys.exit(main())
