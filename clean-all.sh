#!/bin/bash

# clean-all.sh
# Safely removes 'Binaries/', 'Intermediate/', and 'Saved/' folders across all sub-projects.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo "Cleaning sub-projects in: $SCRIPT_DIR"

# Folders to target
TARGETS=("Binaries" "Intermediate" "Saved")

# Use find to locate and remove target directories within the versions/ directory
# -prune ensures we don't descend into a directory we've already matched
for target in "${TARGETS[@]}"; do
    echo "Searching for '$target' folders..."
    find "$SCRIPT_DIR/versions" -type d -name "$target" -prune -print0 | while IFS= read -r -d '' dir; do
        echo "Deleting: $dir"
        rm -rf "$dir"
    completed
completed

echo "Cleanup complete."
