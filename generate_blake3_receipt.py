import subprocess
import json
import glob
import os

files = glob.glob('final_mech_asset/**/*', recursive=True)
files.extend(['winter_protocol_prelude_mecha.usda', 'Agent07_Mecha_Materials.mtlx'])
files.sort()

receipts = []
for f in files:
    if os.path.isfile(f):
        output = subprocess.check_output(['b3sum', f]).decode('utf-8')
        hash_val = output.split()[0]
        receipts.append({
            "file": f,
            "blake3_hash": hash_val
        })

# This is a FLAT packaging manifest over the final_mech_asset/ package. It MUST NOT
# be written to BLAKE3_RECEIPT_CHAIN.json, which is the authoritative prev_hash-linked
# receipt chain owned exclusively by the R6 keystone
# (scripts/verify_r6_delete_resync_replay.py). Writing the flat manifest there clobbers
# the linked chain and breaks end-to-end validate_chain. Write to a distinct path.
output_file = 'final_mech_asset/PACKAGING_MANIFEST.json'
os.makedirs('final_mech_asset', exist_ok=True)
with open(output_file, 'w') as out:
    json.dump({"artifacts": receipts, "receipt_type": "pre-UE4 hero mech asset package", "status": "ADMITTED"}, out, indent=4)
print(f"Packaging manifest generated at {output_file}")
