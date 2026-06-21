#!/usr/bin/env python3
"""SINGLE-SOURCE DRIFT GUARD — crate band literals vs ontology 116.

The morphology ratio bands have ONE authoritative source: ontology/source_law/
116_metric_morphology_bands.ttl (law:*HeightBand bandMinRatio/bandMaxRatio). Any
place that re-states those numbers instead of deriving them is a drift hazard.
The Python gate (scripts/verify_metric_morphology.py::part_bands) now derives them
from the graph, but the Rust crate crates/mech_morphology_law/src/law.rs::class_band
still hardcodes literal tuples. This checker mechanically compares those literals to
116 and exits non-zero on any divergence, so the drift becomes a loud signal instead
of a silent admit-the-wrong-thing bug.

Usage: python3 scripts/check_crate_band_sync.py   (exit 0 = in sync, 1 = drift)
"""
import os
import re
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
TTL = os.path.join(REPO, "ontology", "source_law",
                   "116_metric_morphology_bands.ttl")
LAW_RS = os.path.join(REPO, "crates", "mech_morphology_law", "src", "law.rs")

# PartClass enum (Rust) -> the law:*HeightBand whose appliesToPartClass it mirrors.
# BipedalLeg/Limb and WingArray(span) collapse to one band each in 116.
CLASS_TO_BAND = {
    "MechaCrown": "HeadHeightBand",
    "TorsoSegment": "TorsoHeightBand",
    "BipedalLeg": "LegHeightBand",
    "WingArray": "WingSpanBand",
    "MechaWeapon": "WeaponHeightBand",
}
TOL = 1e-6


def ttl_bands():
    """Parse {bandName: (min, max)} from 116 (rdflib if present, else regex)."""
    text = open(TTL).read()
    out = {}
    # block per "law:<Band> a law:MorphologyBand ; ... bandMinRatio "x" ; bandMaxRatio "y""
    for m in re.finditer(
            r'law:(\w*HeightBand|\w*SpanBand)\b.*?law:bandMinRatio\s+"([\d.]+)".*?'
            r'law:bandMaxRatio\s+"([\d.]+)"', text, re.DOTALL):
        out[m.group(1)] = (float(m.group(2)), float(m.group(3)))
    return out


def crate_bands():
    """Parse {PartClass: (lo, hi)} from law.rs class_band match arms."""
    text = open(LAW_RS).read()
    out = {}
    for m in re.finditer(r'PartClass::(\w+)\s*=>\s*\(([\d.]+),\s*([\d.]+)\)', text):
        out[m.group(1)] = (float(m.group(2)), float(m.group(3)))
    return out


def main():
    ttl = ttl_bands()
    crate = crate_bands()
    drift, ok = [], []
    for cls, band in CLASS_TO_BAND.items():
        want = ttl.get(band)
        got = crate.get(cls)
        if want is None:
            drift.append(f"{cls}: 116 band {band} not found")
        elif got is None:
            drift.append(f"{cls}: not found in crate class_band")
        elif abs(got[0] - want[0]) > TOL or abs(got[1] - want[1]) > TOL:
            drift.append(f"{cls}: crate {got} != 116 {band} {want}")
        else:
            ok.append(f"{cls}: {got} == 116 {band}")
    print("=== crate band sync vs ontology 116 ===")
    for line in ok:
        print("  OK   ", line)
    for line in drift:
        print("  DRIFT", line)
    if drift:
        print(f"\nRESULT: DRIFT ({len(drift)} class band(s) out of sync). "
              "crates/mech_morphology_law/src/law.rs::class_band must be re-synced "
              "to 116 (ideally ggen-generated from it).")
        return 1
    print("\nRESULT: IN SYNC — crate band literals match ontology 116.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
