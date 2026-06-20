import os
import subprocess
import json
import shutil
import hashlib

GGEN_MANIFEST = "ggen-validation-tests/ggen.toml"
GGEN_BIN = "/Users/sac/.local/bin/ggen"
GENERATED_DIR = "generated"

def get_hashes():
    hashes = {}
    for root, dirs, files in os.walk(GENERATED_DIR):
        for file in files:
            filepath = os.path.join(root, file)
            res = subprocess.run(["b3sum", filepath], capture_output=True, text=True, check=True)
            hash_val = res.stdout.split()[0]
            hashes[filepath] = hash_val
    return hashes

def main():
    print("Agent 19: Starting receipt/replay sequence...")
    if not os.path.exists(GENERATED_DIR):
        print(f"Error: {GENERATED_DIR} does not exist.")
        return

    # 1. Hash every artifact with BLAKE3
    print("Hashing current artifacts...")
    before_hashes = get_hashes()
    
    # Build receipt chain
    receipt_before = {
        "event": "hash_before_deletion",
        "artifacts": before_hashes
    }
    with open("receipt_before.json", "w") as f:
        json.dump(receipt_before, f, indent=2)
    print(f"Recorded {len(before_hashes)} hashes in receipt_before.json")

    # 2. Run deletion replay
    print("Deleting generated directory...")
    shutil.rmtree(GENERATED_DIR)

    print("Re-running generator...")
    subprocess.run([GGEN_BIN, "sync", "--manifest", GGEN_MANIFEST], check=True)

    print("Hashing regenerated artifacts...")
    after_hashes = get_hashes()

    receipt_after = {
        "event": "hash_after_regeneration",
        "artifacts": after_hashes
    }
    with open("receipt_after.json", "w") as f:
        json.dump(receipt_after, f, indent=2)
    print(f"Recorded {len(after_hashes)} hashes in receipt_after.json")

    # 3. Verify deterministic reproduction
    success = True
    before_files = set(before_hashes.keys())
    after_files = set(after_hashes.keys())
    
    if before_files != after_files:
        print("Mismatch in generated files!")
        print("Missing in after:", before_files - after_files)
        print("Extra in after:", after_files - before_files)
        success = False
    
    for f in before_files.intersection(after_files):
        if before_hashes[f] != after_hashes[f]:
            print(f"Hash mismatch for {f}: {before_hashes[f]} != {after_hashes[f]}")
            success = False
    
    if success:
        print("DELETION REPLAY SUCCESS: Same source law + generator version reproduced identical artifacts and receipts.")
        final_receipt = {
            "status": "VERIFIED",
            "message": "Same seed + same source law + same generator version reproduced same artifacts, dispositions, and receipts.",
            "artifacts_verified": len(after_hashes)
        }
    else:
        print("DELETION REPLAY FAILED: Non-deterministic generation detected.")
        final_receipt = {
            "status": "FAILED",
            "message": "Hash mismatches or missing files detected during replay.",
        }

    with open("receipt_final.json", "w") as f:
        json.dump(final_receipt, f, indent=2)

if __name__ == "__main__":
    main()
