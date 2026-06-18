#!/usr/bin/env bash
set -e

# cook_html5.sh
# Headless UE4 Pipeline for Rocket Craft
# Takes a .t3d world artifact, injects it into Brm project, and cooks an HTML5 build.

if [ -z "$UE4_ROOT" ]; then
    echo "[!] ERROR: UE4_ROOT environment variable is not set."
    echo "    To manufacture Unreal 4 worlds, you must provide the path to your Unreal Engine 4.24 installation."
    echo "    Example: export UE4_ROOT=/Applications/UnrealEngine-4.24"
    exit 1
fi

PROJECT_DIR="$(pwd)/versions/4.24.0/Brm.uproject"
T3D_SOURCE=$1

if [ -z "$T3D_SOURCE" ]; then
    echo "[!] ERROR: No .t3d source map provided."
    echo "    Usage: $0 <path_to.t3d>"
    exit 1
fi

echo "--- ROCKET CRAFT: WORLD MANUFACTURING PIPELINE ---"
echo "UE4 Root: $UE4_ROOT"
echo "Project: $PROJECT_DIR"
echo "Source Artifact: $T3D_SOURCE"
echo "Target Platform: HTML5 (Shipping)"

# Step 1: Import the .t3d artifact into the project
# Note: In UE4, importing a .t3d into a .umap headless requires the ImportAssets commandlet or a python script.
echo "[1/4] Importing .t3d artifact into Unreal Engine..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    UE4_EDITOR="$UE4_ROOT/Engine/Binaries/Mac/UE4Editor"
else
    UE4_EDITOR="$UE4_ROOT/Engine/Binaries/Linux/UE4Editor"
fi

if [ ! -f "$UE4_EDITOR" ]; then
    echo "[!] ERROR: UE4Editor not found at $UE4_EDITOR"
    exit 1
fi

# We use the ImportAssets commandlet to convert t3d to umap
"$UE4_EDITOR" "$PROJECT_DIR" -run=ImportAssets -source="$T3D_SOURCE" -dest="Game/Content/Maps/ManufacturedWorld" -NoUI -stdout -AllowCommandletRendering

# Step 2: Build the project targets
echo "[2/4] Building C++ Modules..."
"$UE4_ROOT/Engine/Build/BatchFiles/RunUAT.sh" BuildCookRun \
  -project="$PROJECT_DIR" \
  -noP4 -platform=HTML5 -clientconfig=Shipping -build

# Step 3: Cook, Stage, and Package for HTML5
echo "[3/4] Cooking and Packaging HTML5 World..."
"$UE4_ROOT/Engine/Build/BatchFiles/RunUAT.sh" BuildCookRun \
  -project="$PROJECT_DIR" \
  -noP4 -platform=HTML5 -clientconfig=Shipping \
  -cook -stage -package -map=ManufacturedWorld \
  -pak -prereqs -nodebuginfo -targetplatform=HTML5 -utf8output

# Step 4: Transfer to PWA Hosting directory
echo "[4/4] Finalizing World Receipt..."
PACKAGE_DIR="$(pwd)/versions/4.24.0/Saved/StagedBuilds/HTML5"
PWA_DIR="$(pwd)/pwa-staff/manufactured"

mkdir -p "$PWA_DIR"
if [ -d "$PACKAGE_DIR" ]; then
    cp -r "$PACKAGE_DIR/"* "$PWA_DIR/"
    echo "[SUCCESS] Playable browser world manufactured to: $PWA_DIR"
else
    echo "[!] ERROR: Packaging failed. No HTML5 output found."
    exit 1
fi

# Record a receipt
RECEIPT_FILE="$(pwd)/pwa-staff/manufactured/receipt.json"
cat > "$RECEIPT_FILE" <<EOF
{
  "status": "success",
  "engine": "UE4.24",
  "timestamp": "$(date +%s)",
  "artifact": "$T3D_SOURCE",
  "url": "/manufactured/Brm-HTML5-Shipping.html"
}
EOF

echo "World is ready for browser launch."
exit 0
