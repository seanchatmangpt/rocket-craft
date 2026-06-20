#!/usr/bin/env bash

# generate-keystores.sh
# Automates keystore creation for Rocket Craft projects based on README.md parameters.

set -euo pipefail

# Resolve script directory dynamically
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
echo -e "${BOLD}${CYAN}      Rocket Craft Android Keystore Generator       ${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
log_info "Workspace root: ${SCRIPT_DIR}"

# Default passwords matching project configs or overridden via environment variables.
PASS_BRM=${ROCKET_CRAFT_KEYSTORE_PASS:-barbar12}
PASS_ZOMBIE=${ROCKET_CRAFT_KEYSTORE_PASS:-123456654321}
PASS_SHOOTER=${ROCKET_CRAFT_KEYSTORE_PASS:-NIKOLALUKIC}

DNAME="CN=RocketCraft, OU=Dev, O=RocketCraft, L=Unknown, ST=Unknown, C=US"

if ! command -v keytool &> /dev/null; then
    log_error "'keytool' could not be found. Please ensure Java JDK is installed and keytool is in your PATH."
    exit 1
fi

generate_keystore() {
    local KEYSTORE_NAME=$1
    local ALIAS_NAME=$2
    local PASS=$3
    local KEYSTORE_PATH="${SCRIPT_DIR}/${KEYSTORE_NAME}"
    
    if [ -f "${KEYSTORE_PATH}" ]; then
        log_info "Keystore '${KEYSTORE_NAME}' already exists at root. Skipping generation."
    else
        log_info "Generating keystore '${KEYSTORE_NAME}' with alias '${ALIAS_NAME}'..."
        if keytool -genkey -v \
            -keystore "${KEYSTORE_PATH}" \
            -alias "${ALIAS_NAME}" \
            -keyalg RSA \
            -keysize 2048 \
            -validity 10000 \
            -storepass "${PASS}" \
            -keypass "${PASS}" \
            -dname "${DNAME}"; then
            log_success "Successfully generated '${KEYSTORE_NAME}'."
        else
            log_error "Failed to generate '${KEYSTORE_NAME}'."
            return 1
        fi
    fi
}

log_info "Starting keystore generation based on configuration parameters..."

# 1. barbarian-road-mashines-key.keystore (BRM)
generate_keystore "barbarian-road-mashines-key.keystore" "barbarian-road-mashines" "$PASS_BRM"
TARGET_DIR_1="${SCRIPT_DIR}/versions/4.24.0/Build/Android"
mkdir -p "${TARGET_DIR_1}"
cp "${SCRIPT_DIR}/barbarian-road-mashines-key.keystore" "${TARGET_DIR_1}/barbarian-road-mashines-key.keystore"
log_success "Copied BRM keystore to ${TARGET_DIR_1}/"

# 2. zombie-key.keystore (Survival)
generate_keystore "zombie-key.keystore" "zombie" "$PASS_ZOMBIE"
TARGET_DIR_2="${SCRIPT_DIR}/versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/Build/Android"
mkdir -p "${TARGET_DIR_2}"
cp "${SCRIPT_DIR}/zombie-key.keystore" "${TARGET_DIR_2}/zombie-key.keystore"
log_success "Copied SurvivalGame keystore to ${TARGET_DIR_2}/"

# 3. hang3d-nightmare-keystore.keystore (ShooterGame)
generate_keystore "hang3d-nightmare-keystore.keystore" "NIGHTMARE" "$PASS_SHOOTER"
TARGET_DIR_3="${SCRIPT_DIR}/versions/4.24-Shooter/ShooterGame/Build/Android"
mkdir -p "${TARGET_DIR_3}"
cp "${SCRIPT_DIR}/hang3d-nightmare-keystore.keystore" "${TARGET_DIR_3}/hang3d-nightmare-keystore.keystore"
cp "${SCRIPT_DIR}/zombie-key.keystore" "${TARGET_DIR_3}/zombie-key.keystore"
log_success "Copied ShooterGame keystores to ${TARGET_DIR_3}/"

echo -e "${BOLD}${CYAN}----------------------------------------------------${RESET}"
log_success "Keystore generation and setup process completed successfully!"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
