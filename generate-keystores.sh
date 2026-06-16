#!/bin/bash

# generate-keystores.sh
# Automates keystore creation for Rocket Craft projects based on README.md parameters.

# Default passwords matching project configs or overridden via environment variables.
PASS_BRM=${ROCKET_CRAFT_KEYSTORE_PASS:-barbar12}
PASS_ZOMBIE=${ROCKET_CRAFT_KEYSTORE_PASS:-123456654321}
PASS_SHOOTER=${ROCKET_CRAFT_KEYSTORE_PASS:-NIKOLALUKIC}

DNAME="CN=RocketCraft, OU=Dev, O=RocketCraft, L=Unknown, ST=Unknown, C=US"

if ! command -v keytool &> /dev/null; then
    echo "Error: 'keytool' could not be found. Please ensure Java JDK is installed and keytool is in your PATH."
    exit 1
fi

generate_keystore() {
    local KEYSTORE_NAME=$1
    local ALIAS_NAME=$2
    local PASS=$3
    
    if [ -f "$KEYSTORE_NAME" ]; then
        echo "Keystore '$KEYSTORE_NAME' already exists in root directory. Skipping generation."
    else
        echo "Generating keystore '$KEYSTORE_NAME' with alias '$ALIAS_NAME'..."
        keytool -genkey -v \
            -keystore "$KEYSTORE_NAME" \
            -alias "$ALIAS_NAME" \
            -keyalg RSA \
            -keysize 2048 \
            -validity 10000 \
            -storepass "$PASS" \
            -keypass "$PASS" \
            -dname "$DNAME"
        
        if [ $? -eq 0 ]; then
            echo "Successfully generated '$KEYSTORE_NAME'."
        else
            echo "Error: Failed to generate '$KEYSTORE_NAME'."
            return 1
        fi
    fi
}

echo "Starting keystore generation based on README.md parameters..."

# 1. barbarian-road-mashines-key.keystore (BRM)
generate_keystore "barbarian-road-mashines-key.keystore" "barbarian-road-mashines" "$PASS_BRM"
mkdir -p versions/4.24.0/Build/Android
cp "barbarian-road-mashines-key.keystore" "versions/4.24.0/Build/Android/barbarian-road-mashines-key.keystore"
echo "Copied BRM keystore to versions/4.24.0/Build/Android/"

# 2. zombie-key.keystore (Survival)
generate_keystore "zombie-key.keystore" "zombie" "$PASS_ZOMBIE"
mkdir -p versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/Build/Android
cp "zombie-key.keystore" "versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/Build/Android/zombie-key.keystore"
echo "Copied SurvivalGame keystore to versions/4.24-Survival/EpicSurvivalGameSeries-4.24/SurvivalGame/Build/Android/"

# 3. hang3d-nightmare-keystore.keystore (ShooterGame)
generate_keystore "hang3d-nightmare-keystore.keystore" "NIGHTMARE" "$PASS_SHOOTER"
mkdir -p versions/4.24-Shooter/ShooterGame/Build/Android
cp "hang3d-nightmare-keystore.keystore" "versions/4.24-Shooter/ShooterGame/Build/Android/hang3d-nightmare-keystore.keystore"
# Also copy zombie-key.keystore to ShooterGame Build directory just in case it is ever checked
cp "zombie-key.keystore" "versions/4.24-Shooter/ShooterGame/Build/Android/zombie-key.keystore"
echo "Copied ShooterGame keystores to versions/4.24-Shooter/ShooterGame/Build/Android/"

echo "Keystore generation and setup process completed."
