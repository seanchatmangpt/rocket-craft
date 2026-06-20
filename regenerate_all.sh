#!/bin/bash
set -e
python3 scaffold_001c.py
python3 scaffold_ggen_pack.py
python3 generate_mud_slice.py
python3 sync_source_law.py
python3 emit_manufacturing_evidence.py
python3 gen_skeleton.py
python3 split_usda.py
python3 scripts/run_mecha_doe.py
