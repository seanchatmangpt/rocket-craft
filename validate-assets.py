#!/usr/bin/env python3
import os
import subprocess
import sys

# Known missing or problematic assets that should not be referenced in code/config
# These patterns represent assets identified as missing in the ROCKET_CRAFT_AUDIT.md
FORBIDDEN_PATTERNS = [
    "Highrise",
    "Brm-HTML5-Shipping",
]

# Directories to ignore during scanning
IGNORE_DIRS = [
    ".git",
    ".agents",
    "non-project-files",
    "pwa-staff",
    "versions",
    "docs",
]

# Files to ignore (including this script itself and documentation)
IGNORE_FILES = [
    "validate-assets.py",
    "ROCKET_CRAFT_AUDIT.md",
    "README.md",
    "help.md",
]

def validate_assets():
    """
    Greps the codebase for known missing or problematic assets to catch issues early.
    This script serves as a pre-commit or CI check to ensure we don't re-introduce
    references to assets that are known to be missing from the repository.
    """
    print("--- Rocket Craft Asset Validation ---")
    issues_found = False
    
    # Get the root directory of the project
    root_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(root_dir)

    for pattern in FORBIDDEN_PATTERNS:
        print(f"Scanning for missing asset reference: '{pattern}'...")
        try:
            # -r: recursive
            # -n: line number
            # -I: ignore binary files
            cmd = ["grep", "-rnI", pattern, "."]
            result = subprocess.run(cmd, capture_output=True, text=True)
            
            if result.stdout:
                lines = result.stdout.strip().split('\n')
                count = 0
                for line in lines:
                    # Basic filtering for ignored paths
                    skip = False
                    for ignore_dir in IGNORE_DIRS:
                        if f"/{ignore_dir}/" in line or line.startswith(f"./{ignore_dir}/"):
                            skip = True
                            break
                    if skip: continue
                    
                    for ignore_file in IGNORE_FILES:
                        if ignore_file in line:
                            skip = True
                            break
                    if skip: continue
                        
                    print(f"  [!] ALERT: Found reference in {line}")
                    issues_found = True
                    count += 1
                
                if count > 0:
                    print(f"  Found {count} invalid references for '{pattern}'.")
            
        except Exception as e:
            print(f"  [ERROR] Failed to scan for {pattern}: {e}")
            
    print("-" * 44)
    if issues_found:
        print("RESULT: Validation FAILED. Known missing assets are still referenced.")
        print("Action: Remove these references or restore the missing assets.")
        sys.exit(1)
    else:
        print("RESULT: Validation PASSED. No known missing asset references found.")
        sys.exit(0)

if __name__ == "__main__":
    validate_assets()
