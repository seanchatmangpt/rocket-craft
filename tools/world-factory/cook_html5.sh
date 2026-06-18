#!/usr/bin/env bash

# cook_html5.sh
# Headless UE4 Pipeline for Rocket Craft
# Takes a .t3d world artifact, injects it into Brm project, and cooks an HTML5 build.

set -euo pipefail

# Resolve script directory and project root dynamically
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Color support check
if [ -t 1 ] && [ "${NO_COLOR:-}" = "" ]; then
    BOLD="\033[1m"
    RED="\033[31m"
    GREEN="\033[32m"
    YELLOW="\033[33m"
    BLUE="\033[34m"
    CYAN="\033[36m"
    RESET="\033[0m"
else
    BOLD=""
    RED=""
    GREEN=""
    YELLOW=""
    BLUE=""
    CYAN=""
    RESET=""
fi

log_info() { echo -e "${BLUE}${BOLD}[INFO]${RESET} $*"; }
log_success() { echo -e "${GREEN}${BOLD}[SUCCESS]${RESET} $*"; }
log_warn() { echo -e "${YELLOW}${BOLD}[WARN]${RESET} $*"; }
log_error() { echo -e "${RED}${BOLD}[ERROR]${RESET} $*" >&2; }

echo -e "${BOLD}${CYAN}====================================================${RESET}"
echo -e "${BOLD}${CYAN}      Rocket Craft: World Manufacturing Pipeline    ${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"

if [ -z "${UE4_ROOT:-}" ]; then
    log_error "UE4_ROOT environment variable is not set."
    log_warn "To manufacture Unreal 4 worlds, you must provide the path to your Unreal Engine 4.24 installation."
    log_warn "Example: export UE4_ROOT=/Applications/UnrealEngine-4.24"
    exit 1
fi

PROJECT_DIR="${PROJECT_ROOT}/versions/4.24.0/Brm.uproject"
T3D_SOURCE=${1:-""}

if [ -z "${T3D_SOURCE}" ]; then
    log_error "No .t3d source map provided."
    log_warn "Usage: $0 <path_to.t3d>"
    exit 1
fi

log_info "UE4 Root: ${UE4_ROOT}"
log_info "Project: ${PROJECT_DIR}"
log_info "Source Artifact: ${T3D_SOURCE}"
log_info "Target Platform: HTML5 (Shipping)"

# Step 1: Import the .t3d artifact into the project
log_info "[1/4] Importing .t3d artifact into Unreal Engine..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    UE4_EDITOR="${UE4_ROOT}/Engine/Binaries/Mac/UE4Editor"
else
    UE4_EDITOR="${UE4_ROOT}/Engine/Binaries/Linux/UE4Editor"
fi

if [ ! -f "${UE4_EDITOR}" ]; then
    log_error "UE4Editor not found at ${UE4_EDITOR}"
    exit 1
fi

# We use the ImportAssets commandlet to convert t3d to umap
if ! "${UE4_EDITOR}" "${PROJECT_DIR}" -run=ImportAssets -source="${T3D_SOURCE}" -dest="Game/Content/Maps/ManufacturedWorld" -NoUI -stdout -AllowCommandletRendering; then
    log_error "Asset import commandlet failed."
    exit 1
fi

# Step 2: Build the project targets
log_info "[2/4] Building C++ Modules..."
RUN_UAT_SH="${UE4_ROOT}/Engine/Build/BatchFiles/RunUAT.sh"
if [ ! -f "${RUN_UAT_SH}" ]; then
    log_error "RunUAT.sh not found at ${RUN_UAT_SH}"
    exit 1
fi

if ! "${RUN_UAT_SH}" BuildCookRun \
  -project="${PROJECT_DIR}" \
  -noP4 -platform=HTML5 -clientconfig=Shipping -build; then
    log_error "RunUAT build step failed."
    exit 1
fi

# Step 3: Cook, Stage, and Package for HTML5
log_info "[3/4] Cooking and Packaging HTML5 World..."
if ! "${RUN_UAT_SH}" BuildCookRun \
  -project="${PROJECT_DIR}" \
  -noP4 -platform=HTML5 -clientconfig=Shipping \
  -cook -stage -package -map=ManufacturedWorld \
  -pak -prereqs -nodebuginfo -targetplatform=HTML5 -utf8output; then
    log_error "RunUAT cook and packaging step failed."
    exit 1
fi

# Step 4: Transfer to PWA Hosting directory
log_info "[4/4] Finalizing World Receipt..."
PACKAGE_DIR="${PROJECT_ROOT}/versions/4.24.0/Saved/StagedBuilds/HTML5"
PWA_DIR="${PROJECT_ROOT}/pwa-staff/manufactured"

mkdir -p "${PWA_DIR}"
if [ -d "${PACKAGE_DIR}" ]; then
    cp -r "${PACKAGE_DIR}/"* "${PWA_DIR}/"
    log_success "Playable browser world manufactured to: ${PWA_DIR}"
else
    log_error "Packaging failed. No HTML5 output found in ${PACKAGE_DIR}"
    exit 1
fi

# Record a receipt
RECEIPT_FILE="${PWA_DIR}/receipt.json"
cat > "${RECEIPT_FILE}" <<EOF
{
  "status": "success",
  "engine": "UE4.24",
  "timestamp": "$(date +%s)",
  "artifact": "${T3D_SOURCE}",
  "url": "/manufactured/Brm-HTML5-Shipping.html"
}
EOF

log_success "World is ready for browser launch."
echo -e "${BOLD}${CYAN}====================================================${RESET}"
exit 0
