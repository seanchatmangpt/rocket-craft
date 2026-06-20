#!/bin/bash

# Exit immediately if any intermediate command fails, except the validation itself.
set -eo pipefail

TARGET_DIR="/Users/sac/.ggen/packs/ue4_ontology"
GGEN_BIN="/Users/sac/.local/bin/ggen"

echo "=== Starting UE4 Universal RDF Mapping Ontology Validation ==="
echo "Target Directory: $TARGET_DIR"
echo "GGen Binary:      $GGEN_BIN"

# Check if target directory exists
if [ ! -d "$TARGET_DIR" ]; then
    echo "ERROR: Target directory '$TARGET_DIR' does not exist." >&2
    exit 2
fi

# Check if ggen binary exists and is executable
if [ ! -x "$GGEN_BIN" ]; then
    echo "ERROR: GGen binary '$GGEN_BIN' is not executable or does not exist." >&2
    exit 3
fi

# Change directory
echo "Changing directory to '$TARGET_DIR'..."
cd "$TARGET_DIR"

# Execute ggen sync with validation
echo "Running: $GGEN_BIN sync --validate-only true"
echo "--------------------------------------------------"

# Run the command and capture exit code
set +e
"$GGEN_BIN" sync --validate-only true
GGEN_EXIT_CODE=$?
set -e

echo "--------------------------------------------------"
if [ $GGEN_EXIT_CODE -eq 0 ]; then
    echo "SUCCESS: Ontology validation passed."
else
    echo "FAILURE: Ontology validation failed with exit code $GGEN_EXIT_CODE."
fi

exit $GGEN_EXIT_CODE
